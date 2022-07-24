use yakui_core::dom::Dom;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::input::{KeyboardKey, MouseButton};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
use yakui_core::{context, Response};

use crate::style::TextStyle;
use crate::util::widget;
use crate::{colors, icons, pad};

use super::{Pad, RenderTextBox};

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TextBox {
    pub text: String,
    pub style: TextStyle,
    pub padding: Pad,
}

impl TextBox {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
            padding: Pad::all(8.0),
        }
    }

    pub fn show(self) -> Response<TextBoxWidget> {
        widget::<TextBoxWidget>(self)
    }
}

#[derive(Debug)]
pub struct TextBoxWidget {
    props: TextBox,
    updated_text: Option<String>,
    selected: bool,
    cursor: usize,
}

pub struct TextBoxResponse {
    pub text: Option<String>,
}

impl Widget for TextBoxWidget {
    type Props = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        Self {
            props: TextBox::new(""),
            updated_text: None,
            selected: false,
            cursor: 0,
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
        self.selected = context::is_selected();

        let text = self.updated_text.as_ref().unwrap_or(&self.props.text);

        let mut render = RenderTextBox::new(text.clone());
        render.style = self.props.style.clone();
        render.selected = self.selected;
        render.cursor = self.cursor;

        pad(self.props.padding, || {
            render.show();
        });

        Self::Response {
            text: self.updated_text.clone(),
        }
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let layout_node = layout.get(dom.current()).unwrap();
        let mut bg = PaintRect::new(layout_node.rect);
        bg.color = colors::BACKGROUND_3;
        paint.add_rect(bg);

        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }

        if self.selected {
            icons::selection_halo(paint, layout_node.rect);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
            } => {
                context::capture_selection();
                EventResponse::Sink
            }

            WidgetEvent::KeyChanged(key, true) => match key {
                KeyboardKey::Left => {
                    self.move_cursor(-1);
                    EventResponse::Sink
                }

                KeyboardKey::Right => {
                    self.move_cursor(1);
                    EventResponse::Sink
                }

                KeyboardKey::Backspace => {
                    self.delete(-1);
                    EventResponse::Sink
                }

                KeyboardKey::Delete => {
                    self.delete(1);
                    EventResponse::Sink
                }

                KeyboardKey::Home => {
                    self.home();
                    EventResponse::Sink
                }

                KeyboardKey::End => {
                    self.end();
                    EventResponse::Sink
                }

                KeyboardKey::Return => {
                    context::remove_selection();
                    EventResponse::Sink
                }

                KeyboardKey::Escape => {
                    context::remove_selection();
                    EventResponse::Sink
                }
                _ => EventResponse::Bubble,
            },
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

impl TextBoxWidget {
    fn move_cursor(&mut self, delta: i32) {
        let text = self.updated_text.as_ref().unwrap_or(&self.props.text);
        let mut cursor = self.cursor as i32;
        let mut remaining = delta.abs();

        while remaining > 0 {
            cursor = cursor.saturating_add(delta.signum());
            cursor = cursor.min(self.props.text.len() as i32);
            cursor = cursor.max(0);
            self.cursor = cursor as usize;

            if text.is_char_boundary(self.cursor) {
                remaining -= 1;
            }
        }
    }

    fn home(&mut self) {
        self.cursor = 0;
    }

    fn end(&mut self) {
        let text = self.updated_text.as_ref().unwrap_or(&self.props.text);
        self.cursor = text.len();
    }

    fn delete(&mut self, dir: i32) {
        let text = self
            .updated_text
            .get_or_insert_with(|| self.props.text.clone());

        let anchor = self.cursor as i32;
        let mut end = anchor;
        let mut remaining = dir.abs();
        let mut len = 0;

        while remaining > 0 {
            end = end.saturating_add(dir.signum());
            end = end.min(self.props.text.len() as i32);
            end = end.max(0);
            len += 1;

            if text.is_char_boundary(end as usize) {
                remaining -= 1;
            }
        }

        if dir < 0 {
            self.cursor = self.cursor.saturating_sub(len);
        }

        let min = anchor.min(end) as usize;
        let max = anchor.max(end) as usize;
        text.replace_range(min..max, "");
    }
}
