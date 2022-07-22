use std::cell::RefCell;
use std::fmt;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use yakui_core::dom::Dom;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::input::{KeyboardKey, MouseButton};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::Widget;
use yakui_core::{context, Response};

use crate::font::{FontName, Fonts};
use crate::text_renderer::TextGlobalState;
use crate::util::widget;
use crate::{colors, icons};

use super::get_text_layout_size;

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TextBox {
    pub text: String,
    pub color: Color3,
    pub font: FontName,
    pub font_size: f32,
}

impl TextBox {
    pub fn new<S: Into<String>>(font_size: f32, text: S) -> Self {
        Self {
            text: text.into(),
            color: Color3::WHITE,
            font: FontName::new("default"),
            font_size,
        }
    }

    pub fn show(self) -> Response<TextBoxWidget> {
        widget::<TextBoxWidget>(self)
    }
}

pub struct TextBoxWidget {
    props: TextBox,
    updated_text: Option<String>,
    selected: bool,
    cursor: usize,
    cursor_pos: RefCell<Vec2>,
    layout: RefCell<Layout>,
}

pub struct TextBoxResponse {
    pub text: Option<String>,
}

impl Widget for TextBoxWidget {
    type Props = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        let layout = Layout::new(CoordinateSystem::PositiveYDown);
        let cursor = 0;

        Self {
            props: TextBox::new(0.0, ""),
            updated_text: None,
            selected: false,
            cursor,
            cursor_pos: RefCell::new(Vec2::ZERO),
            layout: RefCell::new(layout),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
        self.selected = context::is_selected();

        Self::Response {
            text: self.updated_text.clone(),
        }
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let fonts = dom.get_global_or_init(Fonts::default);
        let font = match fonts.get(&self.props.font) {
            Some(font) => font,
            None => {
                // TODO: Log once that we were unable to find this font.
                return input.min;
            }
        };

        let text = self.updated_text.as_ref().unwrap_or(&self.props.text);

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

        let before_cursor = &text[..self.cursor];
        text_layout.append(&[&*font], &TextStyle::new(before_cursor, font_size, 0));

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

        let after_cursor = &text[self.cursor..];
        text_layout.append(&[&*font], &TextStyle::new(after_cursor, font_size, 0));

        let size = get_text_layout_size(&text_layout, layout.scale_factor());

        input.constrain_min(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let fonts = dom.get_global_or_init(Fonts::default);
        let global = dom.get_global_or_init(TextGlobalState::new);

        let font = match fonts.get(&self.props.font) {
            Some(font) => font,
            None => return,
        };

        let text_layout = self.layout.borrow_mut();
        let mut glyph_cache = global.glyph_cache.borrow_mut();

        glyph_cache.ensure_texture(paint);

        let layout_node = layout.get(dom.current()).unwrap();

        let mut bg = PaintRect::new(layout_node.rect);
        bg.color = colors::BACKGROUND_3;
        paint.add_rect(bg);

        for glyph in text_layout.glyphs() {
            let tex_rect = glyph_cache
                .get_or_insert(paint, &*font, glyph.key)
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

        if self.selected {
            let cursor_pos = layout_node.rect.pos() + *self.cursor_pos.borrow();
            let cursor_size = Vec2::new(1.0, self.props.font_size);

            let mut rect = PaintRect::new(Rect::from_pos_size(cursor_pos, cursor_size));
            rect.color = Color3::RED;
            paint.add_rect(rect);

            icons::selection_halo(paint, layout_node.rect);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseButtonChanged(MouseButton::One, true) => {
                context::capture_selection();
                EventResponse::Sink
            }
            WidgetEvent::KeyChanged(KeyboardKey::Left, true) => {
                let text = self.updated_text.as_ref().unwrap_or(&self.props.text);

                loop {
                    self.cursor = self.cursor.saturating_sub(1);
                    if text.is_char_boundary(self.cursor) {
                        break;
                    }
                }

                EventResponse::Sink
            }
            WidgetEvent::KeyChanged(KeyboardKey::Right, true) => {
                let text = self.updated_text.as_ref().unwrap_or(&self.props.text);

                loop {
                    self.cursor += 1;
                    self.cursor = self.cursor.min(self.props.text.len());
                    if text.is_char_boundary(self.cursor) {
                        break;
                    }
                }

                EventResponse::Sink
            }
            WidgetEvent::KeyChanged(KeyboardKey::Backspace, true) => {
                let text = self
                    .updated_text
                    .get_or_insert_with(|| self.props.text.clone());

                if self.cursor == 0 {
                    return EventResponse::Sink;
                }

                let start = self.cursor - 1;
                let c = text.remove(start);
                self.cursor = self.cursor.saturating_sub(c.len_utf8());
                EventResponse::Sink
            }
            WidgetEvent::KeyChanged(KeyboardKey::Escape, true) => {
                context::remove_selection();
                EventResponse::Sink
            }
            WidgetEvent::TextInput(c) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                let text = self
                    .updated_text
                    .get_or_insert_with(|| self.props.text.clone());

                if text.is_empty() {
                    text.push(*c);
                } else {
                    text.insert(self.cursor, *c);
                }

                self.cursor += c.len_utf8();

                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
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
