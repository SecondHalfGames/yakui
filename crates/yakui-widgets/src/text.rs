use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use yakui_core::context::Context;
use yakui_core::dom::Dom;
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Texture, TextureFormat};
use yakui_core::{Color3, Component, Constraints, Index, Rect, URect, UVec2, Vec2};

#[derive(Debug, Clone)]
struct TextGlobalState {
    default_font: Rc<Font>,
    glyph_cache: Rc<RefCell<GlyphCache>>,
}

#[derive(Debug)]
struct GlyphCache {
    texture: Option<Index>,
    glyphs: HashMap<GlyphRasterConfig, URect>,
    anchors: Vec<UVec2>,
    texture_size: UVec2,
}

impl GlyphCache {
    fn ensure_texture(&mut self, paint: &mut PaintDom) {
        if self.texture.is_none() {
            let texture = paint.create_texture(Texture::new(
                TextureFormat::R8,
                UVec2::new(4096, 4096),
                vec![0; 4096 * 4096],
            ));

            self.texture = Some(texture);
        }
    }

    fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> URect {
        let texture = paint.modify_texture(self.texture.unwrap()).unwrap();

        let others: Vec<_> = self.glyphs.values().copied().collect();

        *self.glyphs.entry(key).or_insert_with(|| {
            let (metrics, bitmap) = font.rasterize_indexed(key.glyph_index, key.px);
            let glyph_size = UVec2::new(metrics.width as u32, metrics.height as u32);

            let anchor_id = self
                .anchors
                .iter()
                .position(|&anchor| {
                    let rect = URect::from_pos_size(anchor, glyph_size);

                    let fits_in_sheet = anchor.x + glyph_size.x < self.texture_size.x
                        && anchor.y + glyph_size.y < self.texture_size.y;

                    // O(n²), oops.
                    let fits_with_others =
                        others.iter().all(|other_rect| !rect.intersects(other_rect));

                    fits_in_sheet && fits_with_others
                })
                .unwrap_or_else(|| {
                    let c = key.glyph_index;
                    let size = key.px;
                    panic!("Tried to add {c}@{size}, but it did not fit.");
                });

            let anchor = self.anchors.swap_remove(anchor_id);
            self.anchors
                .push(UVec2::new(anchor.x + glyph_size.x + 1, anchor.y));
            self.anchors
                .push(UVec2::new(anchor.x, anchor.y + glyph_size.y + 1));

            let size = texture.size();
            blit(anchor, &bitmap, glyph_size, texture.data_mut(), size);

            URect::from_pos_size(anchor, glyph_size)
        })
    }
}

fn get(data: &[u8], size: UVec2, pos: UVec2) -> u8 {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize]
}

fn set(data: &mut [u8], size: UVec2, pos: UVec2, value: u8) {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize] = value;
}

fn blit(
    pos: UVec2,
    source_data: &[u8],
    source_size: UVec2,
    dest_data: &mut [u8],
    dest_size: UVec2,
) {
    for y in pos.y..(pos.y + source_size.y) {
        for x in pos.x..(pos.x + source_size.x) {
            let value = get(source_data, source_size, UVec2::new(x, y));
            set(dest_data, dest_size, UVec2::new(x, y), value);
        }
    }
}

impl TextGlobalState {
    fn new() -> Self {
        let default_font = Font::from_bytes(
            include_bytes!("../assets/Roboto-Regular.ttf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();

        let glyph_cache = GlyphCache {
            texture: None,
            glyphs: HashMap::new(),
            anchors: vec![UVec2::ZERO],

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

#[derive(Debug, Clone)]
pub struct Text {
    pub text: Cow<'static, str>,
    pub font_size: f32,
    global: TextGlobalState,
}

pub struct TextComponent {
    index: Index,
    props: Text,
    layout: RefCell<Layout>,
}

pub type TextResponse = ();

impl Component for TextComponent {
    type Props = Text;
    type Response = TextResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            index,
            props,
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, _layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let global = &self.props.global;

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width: Some(input.max.x),
            max_height: Some(input.max.y),
            ..LayoutSettings::default()
        });

        text_layout.append(
            &[global.default_font.as_ref()],
            &TextStyle::new(&self.props.text, self.props.font_size, 0),
        );

        let mut size = Vec2::ZERO;

        for glyph in text_layout.glyphs() {
            let max = Vec2::new(glyph.x + glyph.width as f32, glyph.y + glyph.height as f32);
            size = size.max(max);
        }

        input.constrain(size)
    }

    fn paint(&self, _dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = self.props.global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(todo!());

        let layout_node = layout.get(self.index).unwrap();
        let viewport = layout.viewport;

        for glyph in text_layout.glyphs() {
            let source_rect =
                glyph_cache.get_or_insert(todo!(), &self.props.global.default_font, glyph.key);

            let size = Vec2::new(glyph.width as f32, glyph.height as f32) / viewport.size();
            let pos = (layout_node.rect.pos() + Vec2::new(glyph.x, glyph.y) + viewport.pos())
                / viewport.size();
            let tex_rect = source_rect
                .as_rect()
                .div_vec2(glyph_cache.texture_size.as_vec2());

            let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
            rect.color = Color3::WHITE;
            rect.texture = Some((todo!(), tex_rect));
            paint.add_rect(rect);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

impl fmt::Debug for TextComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextComponent")
            .field("index", &self.index)
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}

pub fn text<S: Into<Cow<'static, str>>>(font_size: f32, text: S) -> TextResponse {
    let context = Context::active();
    let mut context = context.borrow_mut();
    let dom = context.dom_mut();

    let global = dom
        .get_global_state_or_insert_with::<TextGlobalState, _>(TextGlobalState::new)
        .clone();

    dom.do_component::<TextComponent>(Text {
        text: text.into(),
        font_size,
        global,
    })
}
