use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fontdue::layout::GlyphRasterConfig;
use fontdue::Font;
use yakui_core::geometry::{URect, UVec2};
use yakui_core::paint::{PaintDom, Texture, TextureFormat};
use yakui_core::TextureId;

#[derive(Debug, Clone)]
pub struct TextGlobalState {
    pub glyph_cache: Rc<RefCell<LateBindingGlyphCache>>,
}

/// This is somewhat a default right now
const TEXTURE_SIZE: u32 = 4096;

impl TextGlobalState {
    pub fn new(font_atlas_id: TextureId) -> Self {
        let glyph_cache = LateBindingGlyphCache {
            font_atlas_id,
            font_atlas: Texture::new(
                TextureFormat::R8,
                UVec2::new(TEXTURE_SIZE, TEXTURE_SIZE),
                vec![0; (TEXTURE_SIZE * TEXTURE_SIZE) as usize],
            ),

            glyphs: HashMap::new(),
            next_pos: UVec2::ONE,
            row_height: 0,
        };

        Self {
            glyph_cache: Rc::new(RefCell::new(glyph_cache)),
        }
    }
}

#[derive(Debug)]
pub struct LateBindingGlyphCache {
    font_atlas_id: TextureId,
    font_atlas: Texture,
    glyphs: HashMap<GlyphRasterConfig, URect>,
    next_pos: UVec2,
    row_height: u32,
}

impl LateBindingGlyphCache {
    pub fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> URect {
        *self.glyphs.entry(key).or_insert_with(|| {
            let atlas_size = self.font_atlas.size();

            let (metrics, bitmap) = font.rasterize_indexed(key.glyph_index, key.px);
            let glyph_size = UVec2::new(metrics.width as u32, metrics.height as u32);

            let glyph_max = self.next_pos + glyph_size;
            let pos = if glyph_max.x < atlas_size.x {
                let pos = self.next_pos;
                self.row_height = self.row_height.max(glyph_size.y + 1);
                pos
            } else {
                let pos = UVec2::new(0, self.row_height);
                self.row_height = 0;
                pos
            };
            self.next_pos = pos + UVec2::new(glyph_size.x + 1, 0);

            blit(
                pos,
                &bitmap,
                glyph_size,
                self.font_atlas.data_mut(),
                atlas_size,
            );

            paint.modify_texture(self.font_atlas_id, self.font_atlas.clone());

            URect::from_pos_size(pos, glyph_size)
        })
    }

    pub fn texture_size(&self) -> UVec2 {
        UVec2::new(TEXTURE_SIZE, TEXTURE_SIZE)
    }

    pub fn font_atlas_id(&self) -> TextureId {
        self.font_atlas_id
    }
}

fn get_pixel(data: &[u8], size: UVec2, pos: UVec2) -> u8 {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize]
}

fn set_pixel(data: &mut [u8], size: UVec2, pos: UVec2, value: u8) {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize] = value;
}

pub fn blit(
    dest_pos: UVec2,
    source_data: &[u8],
    source_size: UVec2,
    dest_data: &mut [u8],
    dest_size: UVec2,
) {
    for h in 0..source_size.y {
        for w in 0..source_size.x {
            let pos = UVec2::new(dest_pos.x + w, dest_pos.y + h);

            let value = get_pixel(source_data, source_size, UVec2::new(w, h));
            set_pixel(dest_data, dest_size, pos, value);
        }
    }
}
