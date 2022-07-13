use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fontdue::layout::GlyphRasterConfig;
use fontdue::{Font, FontSettings};
use yakui_core::paint::{PaintDom, Texture, TextureFormat};
use yakui_core::{TextureId, URect, UVec2};

#[derive(Debug, Clone)]
pub struct TextGlobalState {
    pub default_font: Rc<Font>,
    pub glyph_cache: Rc<RefCell<GlyphCache>>,
}

#[derive(Debug)]
pub struct GlyphCache {
    pub texture: Option<TextureId>,
    pub texture_size: UVec2,
    glyphs: HashMap<GlyphRasterConfig, URect>,
    next_pos: UVec2,
    row_height: u32,
}

impl GlyphCache {
    pub fn ensure_texture(&mut self, paint: &mut PaintDom) {
        if self.texture.is_none() {
            let texture = paint.create_texture(Texture::new(
                TextureFormat::R8,
                UVec2::new(4096, 4096),
                vec![0; 4096 * 4096],
            ));

            self.texture = Some(texture);
            self.texture_size = UVec2::new(4096, 4096);
        }
    }

    pub fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> URect {
        *self.glyphs.entry(key).or_insert_with(|| {
            let texture = paint.modify_texture(self.texture.unwrap()).unwrap();

            let (metrics, bitmap) = font.rasterize_indexed(key.glyph_index, key.px);
            let glyph_size = UVec2::new(metrics.width as u32, metrics.height as u32);

            let glyph_max = self.next_pos + glyph_size;
            let pos = if glyph_max.x < self.texture_size.x {
                let pos = self.next_pos;
                self.row_height = self.row_height.max(glyph_size.y + 1);
                pos
            } else {
                let pos = UVec2::new(0, self.row_height);
                self.row_height = 0;
                pos
            };
            self.next_pos = pos + UVec2::new(glyph_size.x + 1, 0);

            let size = texture.size();
            blit(pos, &bitmap, glyph_size, texture.data_mut(), size);

            URect::from_pos_size(pos, glyph_size)
        })
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

impl TextGlobalState {
    pub fn new() -> Self {
        let default_font = Font::from_bytes(
            include_bytes!("../assets/Roboto-Regular.ttf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();

        let glyph_cache = GlyphCache {
            texture: None,
            glyphs: HashMap::new(),
            next_pos: UVec2::ONE,
            row_height: 0,

            // Not initializing to zero to avoid divide by zero issues if we do
            // intialize the texture incorrectly.
            texture_size: UVec2::ONE,
        };

        Self {
            default_font: Rc::new(default_font),
            glyph_cache: Rc::new(RefCell::new(glyph_cache)),
        }
    }
}
