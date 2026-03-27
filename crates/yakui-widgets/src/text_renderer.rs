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
            Kind::Color => TextureFormat::Rgba8Srgb,
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

enum AtlasResult {
    TextureCreationFailed,
    Cached(GlyphRender),
    Inserted(GlyphRender),
    WrongAtlasType(cosmic_text::SwashImage),
}

#[derive(Debug)]
struct InnerAtlas {
    kind: Kind,
    texture: Option<ManagedTextureId>,
    glyph_rects: HashMap<cosmic_text::CacheKey, (URect, Vec2)>,
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
        glyph: &cosmic_text::PhysicalGlyph,
        available_image: Option<cosmic_text::SwashImage>,
    ) -> AtlasResult {
        let Some(texture_id) = self.ensure_texture(paint) else {
            return AtlasResult::TextureCreationFailed;
        };
        let texture_size = paint.texture_mut(texture_id).unwrap().size();

        if let Some((rect, offset)) = self.glyph_rects.get(&glyph.cache_key).cloned() {
            return AtlasResult::Cached(GlyphRender {
                kind: self.kind,
                rect,
                offset,
                tex_rect: rect.as_rect().div_vec2(texture_size.as_vec2()),
                texture: self.texture.unwrap(),
            });
        }

        let image = available_image.unwrap_or_else(|| {
            cache
                .get_image_uncached(font_system, glyph.cache_key)
                .unwrap()
        });

        match image.content {
            cosmic_text::SwashContent::Mask => {
                if self.kind != Kind::Mask {
                    return AtlasResult::WrongAtlasType(image);
                }
            }
            cosmic_text::SwashContent::Color => {
                if self.kind != Kind::Color {
                    return AtlasResult::WrongAtlasType(image);
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

        self.glyph_rects.insert(glyph.cache_key, (rect, offset));

        AtlasResult::Inserted(GlyphRender {
            kind: self.kind,
            rect,
            offset,
            tex_rect: rect.as_rect().div_vec2(texture_size.as_vec2()),
            texture: self.texture.unwrap(),
        })
    }

    fn clear(&mut self) {
        self.glyph_rects.clear();
        self.next_pos = UVec2::ZERO;
        self.max_height = 0;
    }
}

#[inline]
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
    color_atlas: InnerAtlas,
    mask_atlas: InnerAtlas,
    swash: cosmic_text::SwashCache,
    glyph_kind_cache: HashMap<cosmic_text::CacheKey, Kind>,
}

impl TextAtlas {
    /// Creates a new [`TextAtlas`] with the given [`ColorMode`].
    pub fn new() -> Self {
        let color_atlas = InnerAtlas::new(Kind::Color);
        let mask_atlas = InnerAtlas::new(Kind::Mask);

        Self {
            color_atlas,
            mask_atlas,
            swash: cosmic_text::SwashCache::new(),
            glyph_kind_cache: HashMap::new(),
        }
    }

    pub fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font_system: &mut cosmic_text::FontSystem,
        glyph: &cosmic_text::LayoutGlyph,
    ) -> GlyphRender {
        if glyph.color_opt.is_some() {
            panic!("Glyph should not have color_opt! yakui uses its own color.");
        }
        let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

        if let Some(kind) = self.glyph_kind_cache.get(&physical_glyph.cache_key) {
            let result = match kind {
                Kind::Mask => self.mask_atlas.get_or_insert(
                    paint,
                    font_system,
                    &mut self.swash,
                    &physical_glyph,
                    None,
                ),
                Kind::Color => self.color_atlas.get_or_insert(
                    paint,
                    font_system,
                    &mut self.swash,
                    &physical_glyph,
                    None,
                ),
            };

            match result {
                AtlasResult::TextureCreationFailed => {
                    panic!("Failed to create texture for text atlas.")
                }
                AtlasResult::Cached(glyph_render) => glyph_render,
                // mismatch of `kind`
                AtlasResult::Inserted(..) | AtlasResult::WrongAtlasType(..) => {
                    panic!("Font changed during runtime and TextAtlas cache wasn't reset.");
                }
            }
        } else {
            match self.mask_atlas.get_or_insert(
                paint,
                font_system,
                &mut self.swash,
                &physical_glyph,
                None,
            ) {
                AtlasResult::TextureCreationFailed => {
                    panic!("Failed to create texture for text atlas.")
                }
                AtlasResult::Cached(glyph_render) | AtlasResult::Inserted(glyph_render) => {
                    self.glyph_kind_cache
                        .insert(physical_glyph.cache_key, glyph_render.kind);
                    glyph_render
                }
                AtlasResult::WrongAtlasType(image) => match image.content {
                    cosmic_text::SwashContent::Mask => unreachable!(),
                    cosmic_text::SwashContent::Color => {
                        match self.color_atlas.get_or_insert(
                            paint,
                            font_system,
                            &mut self.swash,
                            &physical_glyph,
                            Some(image),
                        ) {
                            AtlasResult::Cached(glyph_render)
                            | AtlasResult::Inserted(glyph_render) => {
                                self.glyph_kind_cache
                                    .insert(physical_glyph.cache_key, glyph_render.kind);
                                glyph_render
                            }
                            _ => panic!("Cannot cache glyph."),
                        }
                    }
                    cosmic_text::SwashContent::SubpixelMask => {
                        panic!("yakui does not support SubpixelMask glyph content!")
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
struct InnerTextState {
    atlas: TextAtlas,
}

impl InnerTextState {
    fn fonts_changed(&mut self) {
        self.atlas.mask_atlas.clear();
        self.atlas.color_atlas.clear();
        self.atlas.glyph_kind_cache.clear();
    }
}

#[derive(Debug, Clone)]
pub struct TextGlobalState {
    inner: Rc<RefCell<InnerTextState>>,
}

impl TextGlobalState {
    /// This function should be called whenever there's a change to the fonts loaded.
    pub fn fonts_changed(&self) {
        self.inner.borrow_mut().fonts_changed()
    }

    pub fn get_glyph_render(
        &self,
        paint: &mut PaintDom,
        font_system: &mut cosmic_text::FontSystem,
        glyph: &cosmic_text::LayoutGlyph,
    ) -> GlyphRender {
        self.inner
            .borrow_mut()
            .atlas
            .get_or_insert(paint, font_system, glyph)
    }

    pub fn new() -> Self {
        let inner = InnerTextState {
            atlas: TextAtlas::new(),
        };

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }
}
