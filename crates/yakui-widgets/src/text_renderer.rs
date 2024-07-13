use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fontdue::layout::GlyphRasterConfig;
use fontdue::Font;
use yakui_core::geometry::{URect, UVec2};
use yakui_core::paint::{PaintDom, Texture, TextureFormat};
use yakui_core::ManagedTextureId;

#[cfg(not(target_arch = "wasm32"))]
const TEXTURE_SIZE: u32 = 4096;

// When targeting the web, limit the texture atlas size to 2048x2048 to fit into
// WebGL 2's limitations. In the future, we should introduce a way to query for
// these limits.
#[cfg(target_arch = "wasm32")]
const TEXTURE_SIZE: u32 = 2048;

#[derive(Debug, Clone)]
pub struct TextGlobalState {
    pub glyph_cache: Rc<RefCell<GlyphCache>>,
}

#[derive(Debug)]
pub struct GlyphCache {
    pub texture: Option<ManagedTextureId>,
    pub texture_size: UVec2,
    glyphs: HashMap<GlyphRasterConfig, URect>,
    next_pos: UVec2,
    max_height: u32,
}

impl GlyphCache {
    pub fn ensure_texture(&mut self, paint: &mut PaintDom) {
        if self.texture.is_none() {
            let texture = paint.add_texture(Texture::new(
                TextureFormat::R8,
                UVec2::new(TEXTURE_SIZE, TEXTURE_SIZE),
                vec![0; (TEXTURE_SIZE * TEXTURE_SIZE) as usize],
            ));

            self.texture = Some(texture);
            self.texture_size = UVec2::new(TEXTURE_SIZE, TEXTURE_SIZE);
        }
    }

    pub fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> URect {
        *self.glyphs.entry(key).or_insert_with(|| {
            paint.mark_texture_modified(self.texture.unwrap());
            let texture = paint.texture_mut(self.texture.unwrap()).unwrap();

            let (metrics, bitmap) = font.rasterize_indexed(key.glyph_index, key.px);
            let glyph_size = UVec2::new(metrics.width as u32, metrics.height as u32);

            let glyph_max = self.next_pos + glyph_size;
            let pos = if glyph_max.x < self.texture_size.x {
                self.next_pos
            } else {
                UVec2::new(0, self.max_height)
            };

            self.max_height = self.max_height.max(pos.y + glyph_size.y + 1);
            self.next_pos = pos + UVec2::new(glyph_size.x + 1, 0);

            let size = texture.size();
            blit(pos, glyph_size, &bitmap, size, texture.data_mut());

            URect::from_pos_size(pos, glyph_size)
        })
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

impl TextGlobalState {
    pub fn new() -> Self {
        let glyph_cache = GlyphCache {
            texture: None,
            glyphs: HashMap::new(),
            next_pos: UVec2::ONE,
            max_height: 0,

            // Not initializing to zero to avoid divide by zero issues if we do
            // intialize the texture incorrectly.
            texture_size: UVec2::ONE,
        };

        Self {
            glyph_cache: Rc::new(RefCell::new(glyph_cache)),
        }
    }
}
