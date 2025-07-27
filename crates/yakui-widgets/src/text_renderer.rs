use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use yakui_core::geometry::{Rect, URect, UVec2, Vec2};
use yakui_core::paint::{PaintDom, Texture, TextureFilter, TextureFormat};
use yakui_core::ManagedTextureId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Mask,
    Color,
}

impl Kind {
    fn num_channels(self) -> usize {
        match self {
            Kind::Mask => 1,
            Kind::Color => 4,
        }
    }

    fn texture_format(self) -> TextureFormat {
        match self {
            Kind::Mask => TextureFormat::R8,
            Kind::Color => TextureFormat::Rgba8SrgbPremultiplied,
        }
    }
}

pub struct GlyphRender {
    pub(crate) kind: Kind,
    pub rect: URect,
    pub offset: Vec2,
    pub tex_rect: Rect,
    pub texture: ManagedTextureId,
}

#[derive(Debug)]
pub struct InnerAtlas {
    pub(crate) kind: Kind,
    pub texture: Option<ManagedTextureId>,
    pub glyph_rects: HashMap<cosmic_text::CacheKey, (URect, Vec2)>,
    next_pos: UVec2,
    max_height: u32,
}

impl InnerAtlas {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            texture: None,
            glyph_rects: HashMap::new(),
            next_pos: UVec2::ZERO,
            max_height: 0,
        }
    }

    fn ensure_texture(&mut self, paint: &mut PaintDom) -> Option<ManagedTextureId> {
        let texture_size = paint.limits()?.max_texture_size_2d.min(4096);

        if self.texture.is_none() {
            let mut texture = Texture::new(
                self.kind.texture_format(),
                UVec2::new(texture_size, texture_size),
                vec![0; (texture_size * texture_size) as usize * self.kind.num_channels()],
            );
            texture.mag_filter = TextureFilter::Linear;
            texture.min_filter = TextureFilter::Linear;
            self.texture = Some(paint.add_texture(texture))
        }

        self.texture
    }

    fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font_system: &mut cosmic_text::FontSystem,
        cache: &mut cosmic_text::SwashCache,
        glyph: &cosmic_text::LayoutGlyph,
        image: Option<cosmic_text::SwashImage>,
    ) -> Result<Option<GlyphRender>, Option<cosmic_text::SwashImage>> {
        let Some(texture_id) = self.ensure_texture(paint) else {
            return Ok(None);
        };

        let texture_size = paint.texture_mut(texture_id).unwrap().size();

        let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
        if let Some((rect, offset)) = self.glyph_rects.get(&physical_glyph.cache_key).cloned() {
            return Ok(Some(GlyphRender {
                kind: self.kind,
                rect,
                offset,
                tex_rect: rect.as_rect().div_vec2(texture_size.as_vec2()),
                texture: self.texture.unwrap(),
            }));
        }

        if glyph.color_opt.is_some() {
            panic!("glyph should not have color_opt! yakui uses its own color.");
        }

        let Some(image) =
            image.or_else(|| cache.get_image_uncached(font_system, physical_glyph.cache_key))
        else {
            return Err(None);
        };

        match image.content {
            cosmic_text::SwashContent::Mask => {
                if self.kind != Kind::Mask {
                    return Err(Some(image));
                }
            }
            cosmic_text::SwashContent::Color => {
                if self.kind != Kind::Color {
                    return Err(Some(image));
                }
            }
            cosmic_text::SwashContent::SubpixelMask => {
                panic!("yakui does not support SubpixelMask glyph content!")
            }
        }

        let glyph_size = UVec2::new(image.placement.width, image.placement.height);

        let pos = if (self.next_pos + glyph_size).x < texture_size.x {
            self.next_pos
        } else {
            UVec2::new(0, self.max_height)
        };

        let glyph_max = pos + glyph_size;
        if glyph_max.x >= texture_size.x || glyph_max.y >= texture_size.y {
            panic!("Overflowed glyph cache!");
        }

        self.max_height = self.max_height.max(pos.y + glyph_size.y + 1);
        self.next_pos = pos + UVec2::new(glyph_size.x + 1, 0);

        let num_channels = self.kind.num_channels() as u32;
        let scale = UVec2::new(num_channels, 1);
        blit(
            pos * scale,
            glyph_size * scale,
            &image.data,
            texture_size * scale,
            paint.texture_mut(self.texture.unwrap()).unwrap().data_mut(),
        );
        paint.mark_texture_modified(self.texture.unwrap());

        let rect = URect::from_pos_size(pos, glyph_size);
        let offset = Vec2::new(image.placement.left as f32, image.placement.top as f32);

        self.glyph_rects
            .insert(physical_glyph.cache_key, (rect, offset));

        Ok(Some(GlyphRender {
            kind: self.kind,
            rect,
            offset,
            tex_rect: rect.as_rect().div_vec2(texture_size.as_vec2()),
            texture: self.texture.unwrap(),
        }))
    }

    #[allow(dead_code)] // we currently never remove textures
    fn clear(&mut self, paint: &mut PaintDom) {
        self.glyph_rects.clear();
        self.next_pos = UVec2::ZERO;
        self.max_height = 0;

        if let Some(id) = self.texture.take() {
            paint.remove_texture(id);
        }
    }
}

fn blit(pos: UVec2, src_size: UVec2, src: &[u8], dst_size: UVec2, dst: &mut [u8]) {
    debug_assert!(dst_size.x >= src_size.x);
    debug_assert!(dst_size.y >= src_size.y);

    for row in 0..src_size.y {
        let y1 = row;
        let s1 = y1 * src_size.x;
        let e1 = s1 + src_size.x;

        let y2 = row + pos.y;
        let s2 = y2 * dst_size.x + pos.x;
        let e2 = s2 + src_size.x;

        dst[s2 as usize..e2 as usize].copy_from_slice(&src[s1 as usize..e1 as usize])
    }
}

/// An atlas containing a cache of rasterized glyphs that can be rendered.
#[derive(Debug)]
pub struct TextAtlas {
    pub(crate) color_atlas: InnerAtlas,
    pub(crate) mask_atlas: InnerAtlas,
}

impl TextAtlas {
    /// Creates a new [`TextAtlas`] with the given [`ColorMode`].
    pub fn new() -> Self {
        let color_atlas = InnerAtlas::new(Kind::Color);
        let mask_atlas = InnerAtlas::new(Kind::Mask);

        Self {
            color_atlas,
            mask_atlas,
        }
    }
}

#[derive(Debug)]
pub struct InnerState {
    pub atlas: TextAtlas,
    pub swash: cosmic_text::SwashCache,
}

impl InnerState {
    pub fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font_system: &mut cosmic_text::FontSystem,
        glyph: &cosmic_text::LayoutGlyph,
    ) -> Option<GlyphRender> {
        let a =
            self.atlas
                .mask_atlas
                .get_or_insert(paint, font_system, &mut self.swash, glyph, None);

        match a {
            Ok(glyph) => glyph,
            Err(image) => {
                let b = self.atlas.color_atlas.get_or_insert(
                    paint,
                    font_system,
                    &mut self.swash,
                    glyph,
                    image,
                );

                b.ok()?
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextGlobalState {
    pub inner: Rc<RefCell<InnerState>>,
}

impl TextGlobalState {
    pub fn get_or_insert(
        &self,
        paint: &mut PaintDom,
        font_system: &mut cosmic_text::FontSystem,
        glyph: &cosmic_text::LayoutGlyph,
    ) -> Option<GlyphRender> {
        self.inner
            .borrow_mut()
            .get_or_insert(paint, font_system, glyph)
    }

    pub fn new() -> Self {
        let state = InnerState {
            swash: cosmic_text::SwashCache::new(),
            atlas: TextAtlas::new(),
        };

        Self {
            inner: Rc::new(RefCell::new(state)),
        }
    }
}
