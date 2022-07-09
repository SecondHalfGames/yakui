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
use yakui_core::paint::{self, PaintRect, Texture};
use yakui_core::{Color3, Component, Constraints, Index, Rect, Vec2};

#[derive(Debug, Clone)]
struct TextGlobalState {
    default_font: Rc<Font>,
    glyph_cache: Rc<RefCell<GlyphCache>>,
}

#[derive(Debug)]
struct GlyphCache {
    texture: Option<Index>,
    letters: HashMap<GlyphRasterConfig, Rect>,
    taken_space: Vec2,
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
            letters: HashMap::new(),
            taken_space: Vec2::ZERO,
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

    fn paint(&self, _dom: &Dom, layout: &LayoutDom, output: &mut paint::Output) {
        let text_layout = self.layout.borrow_mut();
        let glyph_cache = self.props.global.glyph_cache.borrow_mut();

        let layout_node = layout.get(self.index).unwrap();
        let viewport = layout.viewport;
        let bounding = layout_node.rect;

        for glyph in text_layout.glyphs() {
            let size = Vec2::new(glyph.width as f32, glyph.height as f32) / viewport.size();
            let pos = (layout_node.rect.pos() + Vec2::new(glyph.x, glyph.y) + viewport.pos())
                / viewport.size();

            let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
            rect.color = Color3::WHITE;
            rect.texture = None; // TODO
            output.add_rect(rect);
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
