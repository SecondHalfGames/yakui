use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle as FontdueTextStyle};
use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::font::Fonts;
use crate::style::TextStyle;
use crate::text_renderer::TextGlobalState;
use crate::util::widget;

use super::get_text_layout_size;

/**
Text that can be edited.

Responds with [RenderTextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RenderTextBox {
    pub text: String,
    pub style: TextStyle,
    pub selected: bool,
    pub cursor: usize,
}

impl RenderTextBox {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
            selected: false,
            cursor: 0,
        }
    }

    pub fn show(self) -> Response<RenderTextBoxWidget> {
        widget::<RenderTextBoxWidget>(self)
    }
}

pub struct RenderTextBoxWidget {
    props: RenderTextBox,
    cursor_pos: RefCell<Vec2>,
    layout: RefCell<Layout>,
}

pub type RenderTextBoxResponse = ();

impl Widget for RenderTextBoxWidget {
    type Props = RenderTextBox;
    type Response = RenderTextBoxResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            props: RenderTextBox::new(""),
            cursor_pos: RefCell::new(Vec2::ZERO),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let fonts = dom.get_global_or_init(Fonts::default);
        let font = match fonts.get(&self.props.style.font) {
            Some(font) => font,
            None => {
                // TODO: Log once that we were unable to find this font.
                return input.min;
            }
        };

        let text = &self.props.text;

        let (max_width, max_height) = if input.is_bounded() {
            (
                Some(input.max.x * layout.scale_factor()),
                Some(input.max.y * layout.scale_factor()),
            )
        } else {
            (None, None)
        };

        let font_size = self.props.style.font_size * layout.scale_factor();

        let mut text_layout = self.layout.borrow_mut();
        text_layout.reset(&LayoutSettings {
            max_width,
            max_height,
            ..LayoutSettings::default()
        });

        let before_cursor = &text[..self.props.cursor];
        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(before_cursor, font_size, 0),
        );

        let cursor_y = text_layout
            .lines()
            .and_then(|lines| lines.last())
            .map(|line| line.baseline_y - line.max_ascent)
            .unwrap_or_default();
        let cursor_x = text_layout
            .glyphs()
            .last()
            .map(|glyph| glyph.x + glyph.width as f32 + 1.0)
            .unwrap_or_default();
        let cursor_pos = Vec2::new(cursor_x, cursor_y) / layout.scale_factor();
        *self.cursor_pos.borrow_mut() = cursor_pos;

        let after_cursor = &text[self.props.cursor..];
        text_layout.append(
            &[&*font],
            &FontdueTextStyle::new(after_cursor, font_size, 0),
        );

        let size = get_text_layout_size(&text_layout, layout.scale_factor());

        input.constrain_min(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let fonts = dom.get_global_or_init(Fonts::default);
        let global = dom.get_global_or_init(TextGlobalState::new);

        let font = match fonts.get(&self.props.style.font) {
            Some(font) => font,
            None => return,
        };

        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(paint);

        let layout_node = layout.get(dom.current()).unwrap();

        for glyph in text_layout.glyphs() {
            let tex_rect = glyph_cache
                .get_or_insert(paint, &*font, glyph.key)
                .as_rect()
                .div_vec2(glyph_cache.texture_size.as_vec2());

            let size = Vec2::new(glyph.width as f32, glyph.height as f32) / layout.scale_factor();
            let pos = layout_node.rect.pos() + Vec2::new(glyph.x, glyph.y) / layout.scale_factor();

            let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
            rect.color = self.props.style.color;
            rect.texture = Some((glyph_cache.texture.unwrap(), tex_rect));
            rect.pipeline = Pipeline::Text;
            paint.add_rect(rect);
        }

        if self.props.selected {
            let cursor_pos = layout_node.rect.pos() + *self.cursor_pos.borrow();
            let cursor_size = Vec2::new(1.0, self.props.style.font_size);

            let mut rect = PaintRect::new(Rect::from_pos_size(cursor_pos, cursor_size));
            rect.color = Color3::RED;
            paint.add_rect(rect);
        }
    }
}

impl fmt::Debug for RenderTextBoxWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderTextBoxWidget")
            .field("props", &self.props)
            .field("layout", &"(no debug impl)")
            .finish()
    }
}
