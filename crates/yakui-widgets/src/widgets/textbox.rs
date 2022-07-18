use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::colors;
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

#[derive(Debug, Clone)]
pub struct TextBox {
    pub text: Cow<'static, str>,
    pub color: Color3,
    pub font_size: f32,
}

impl TextBox {
    pub fn new(font_size: f32, text: Cow<'static, str>) -> Self {
        Self {
            text,
            color: Color3::WHITE,
            font_size,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            color: Color3::WHITE,
            font_size: 14.0,
        }
    }

    pub fn show(self) -> Response<TextBoxWidget> {
        widget::<TextBoxWidget>(self)
    }
}

pub struct TextBoxWidget {
    props: TextBox,
    focused: bool,
    cursor: usize,
    cursor_pos: RefCell<Vec2>,
    layout: RefCell<Layout>,
}

pub struct TextBoxResponse {
    pub text: Cow<'static, str>,
}

impl Widget for TextBoxWidget {
    type Props = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);
        let cursor = 0;

        Self {
            props: TextBox::new(0.0, Cow::Borrowed("")),
            focused: false,
            cursor,
            cursor_pos: RefCell::new(Vec2::ZERO),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;

        Self::Response {
            text: self.props.text.clone(),
        }
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let global = dom.get_global_or_init(TextGlobalState::new);

        let (max_width, max_height) = if input.is_bounded() {
            (
                Some(input.max.x * layout.scale_factor()),
                Some(input.max.y * layout.scale_factor()),
            )
        } else {
            (None, None)
        };

        let font_size = self.props.font_size * layout.scale_factor();

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            ..LayoutSettings::default()
        });

        let before_cursor = &self.props.text[..self.cursor];
        text_layout.append(
            &[global.default_font.as_ref()],
            &TextStyle::new(before_cursor, font_size, 0),
        );

        let cursor_pos = text_layout
            .glyphs()
            .last()
            .map(|glyph| {
                Vec2::new(glyph.x + glyph.width as f32 + 1.0, glyph.y) / layout.scale_factor()
            })
            .unwrap_or(Vec2::ZERO);
        *self.cursor_pos.borrow_mut() = cursor_pos;

        let after_cursor = &self.props.text[self.cursor..];
        text_layout.append(
            &[global.default_font.as_ref()],
            &TextStyle::new(after_cursor, font_size, 0),
        );

        let mut size = Vec2::ZERO;

        for glyph in text_layout.glyphs() {
            let max = Vec2::new(glyph.x + glyph.width as f32, glyph.y + glyph.height as f32)
                / layout.scale_factor();
            size = size.max(max);
        }

        input.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let global = dom.get_global_or_init(TextGlobalState::new);

        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(paint);

        let layout_node = layout.get(dom.current()).unwrap();

        let mut bg = PaintRect::new(layout_node.rect);
        bg.color = colors::BACKGROUND_3;
        paint.add_rect(bg);

        for glyph in text_layout.glyphs() {
            let tex_rect = glyph_cache
                .get_or_insert(paint, &global.default_font, glyph.key)
                .as_rect()
                .div_vec2(glyph_cache.texture_size.as_vec2());

            let size = Vec2::new(glyph.width as f32, glyph.height as f32) / layout.scale_factor();
            let pos = layout_node.rect.pos() + Vec2::new(glyph.x, glyph.y) / layout.scale_factor();

            let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
            rect.color = self.props.color;
            rect.texture = Some((glyph_cache.texture.unwrap(), tex_rect));
            rect.pipeline = Pipeline::Text;
            paint.add_rect(rect);
        }

        let cursor_pos = layout_node.rect.pos() + *self.cursor_pos.borrow();
        let cursor_size = Vec2::new(1.0, self.props.font_size);

        let mut rect = PaintRect::new(Rect::from_pos_size(cursor_pos, cursor_size));
        rect.color = Color3::RED;
        paint.add_rect(rect);
    }
}

impl fmt::Debug for TextBoxWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextBoxWidget")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}
