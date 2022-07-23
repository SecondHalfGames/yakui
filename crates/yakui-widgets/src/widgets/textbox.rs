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
