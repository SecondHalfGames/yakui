use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use fontdue::layout::{Layout, LinePosition};
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::input::{KeyCode, MouseButton};
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::ignore_debug::IgnoreDebug;
use crate::shapes::RoundedRectangle;
use crate::style::TextStyle;
use crate::util::widget;
use crate::{colors, pad, shapes};

use super::{Pad, RenderTextBox};

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct TextBox {
    pub text: String,
    pub style: TextStyle,
    pub padding: Pad,
    pub fill: Option<Color>,
    /// Drawn when no text has been set
    pub placeholder: String,
}

impl TextBox {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
            padding: Pad::all(8.0),
            fill: Some(colors::BACKGROUND_3),
            placeholder: String::new(),
        }
    }

    pub fn show(self) -> Response<TextBoxResponse> {
        widget::<TextBoxWidget>(self)
    }
}

#[derive(Debug)]
pub struct TextBoxWidget {
    props: TextBox,
    updated_text: Option<String>,
    selected: bool,
    cursor: usize,
    text_layout: Option<IgnoreDebug<Rc<RefCell<Layout>>>>,
    activated: bool,
    lost_focus: bool,
}

pub struct TextBoxResponse {
    pub text: Option<String>,
    /// Whether the user pressed "Enter" in this box
    pub activated: bool,
    /// Whether the box lost focus
    pub lost_focus: bool,
}

impl Widget for TextBoxWidget {
    type Props<'a> = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        Self {
            props: TextBox::new(""),
            updated_text: None,
            selected: false,
            cursor: 0,
            text_layout: None,
            activated: false,
            lost_focus: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut text = self.updated_text.as_ref().unwrap_or(&self.props.text);
        let use_placeholder = text.is_empty();
        if use_placeholder {
            text = &self.props.placeholder;
        }

        // Make sure the cursor is within bounds if the text has changed
        self.cursor = self.cursor.min(text.len());

        let mut render = RenderTextBox::new(text.clone());
        render.style = self.props.style.clone();
        render.selected = self.selected;
        if !use_placeholder {
            render.cursor = self.cursor;
        }
        if use_placeholder {
            // Dim towards background
            render.style.color = self
                .props
                .style
                .color
                .lerp(&self.props.fill.unwrap_or(Color::CLEAR), 0.75);
        }

        pad(self.props.padding, || {
            let res = render.show();
            self.text_layout = Some(IgnoreDebug(res.into_inner().layout));
        });

        Self::Response {
            text: self.updated_text.take(),
            activated: mem::take(&mut self.activated),
            lost_focus: mem::take(&mut self.lost_focus),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        self.default_layout(ctx, constraints)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        if let Some(fill_color) = self.props.fill {
            let mut bg = RoundedRectangle::new(layout_node.rect, 6.0);
            bg.color = fill_color;
            bg.add(ctx.paint);
        }

        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }

        if self.selected {
            shapes::selection_halo(ctx.paint, layout_node.rect);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::FocusChanged(focused) => {
                self.selected = *focused;
                if !*focused {
                    self.lost_focus = true;
                }
                EventResponse::Sink
            }

            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                inside: true,
                down,
                position,
                ..
            } => {
                if !down {
                    return EventResponse::Sink;
                }

                ctx.input.set_selection(Some(ctx.dom.current()));

                if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                    if let Some(text_layout) = &self.text_layout {
                        let text_layout = text_layout.borrow();

                        let scale_factor = ctx.layout.scale_factor();
                        let relative_pos =
                            *position - layout.rect.pos() - self.props.padding.offset();
                        let glyph_pos = relative_pos * scale_factor;

                        let Some(line) = pick_text_line(&text_layout, glyph_pos.y) else {
                            return EventResponse::Sink;
                        };

                        self.cursor = pick_character_on_line(
                            &text_layout,
                            line.glyph_start,
                            line.glyph_end,
                            glyph_pos.x,
                        );
                    }
                }

                EventResponse::Sink
            }

            WidgetEvent::KeyChanged { key, down, .. } => match key {
                KeyCode::ArrowLeft => {
                    if *down {
                        self.move_cursor(-1);
                    }
                    EventResponse::Sink
                }

                KeyCode::ArrowRight => {
                    if *down {
                        self.move_cursor(1);
                    }
                    EventResponse::Sink
                }

                KeyCode::Backspace => {
                    if *down {
                        self.delete(-1);
                    }
                    EventResponse::Sink
                }

                KeyCode::Delete => {
                    if *down {
                        self.delete(1);
                    }
                    EventResponse::Sink
                }

                KeyCode::Home => {
                    if *down {
                        self.home();
                    }
                    EventResponse::Sink
                }

                KeyCode::End => {
                    if *down {
                        self.end();
                    }
                    EventResponse::Sink
                }

                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if *down {
                        ctx.input.set_selection(None);
                        self.activated = true;
                    }
                    EventResponse::Sink
                }

                KeyCode::Escape => {
                    if *down {
                        ctx.input.set_selection(None);
                    }
                    EventResponse::Sink
                }
                _ => EventResponse::Sink,
            },
            WidgetEvent::TextInput(c) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                let text = self
                    .updated_text
                    .get_or_insert_with(|| self.props.text.clone());

                // Before trying to input text, make sure that our cursor fits
                // in the string and is not in the middle of a codepoint!
                self.cursor = self.cursor.min(text.len());
                while !text.is_char_boundary(self.cursor) {
                    self.cursor = self.cursor.saturating_sub(1);
                }

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

fn pick_text_line(layout: &Layout, pos_y: f32) -> Option<&LinePosition> {
    let lines = layout.lines()?;

    let mut closest_line = 0;
    let mut closest_line_dist = f32::INFINITY;
    for (index, line) in lines.iter().enumerate() {
        let dist = (pos_y - line.baseline_y).abs();
        if dist < closest_line_dist {
            closest_line = index;
            closest_line_dist = dist;
        }
    }

    lines.get(closest_line)
}

fn pick_character_on_line(
    layout: &Layout,
    line_glyph_start: usize,
    line_glyph_end: usize,
    pos_x: f32,
) -> usize {
    let mut closest_byte_offset = 0;
    let mut closest_dist = f32::INFINITY;

    let possible_positions = layout
        .glyphs()
        .iter()
        .skip(line_glyph_start)
        .take(line_glyph_end + 1 - line_glyph_start)
        .flat_map(|glyph| {
            let before = Vec2::new(glyph.x, glyph.y);
            let after = Vec2::new(glyph.x + glyph.width as f32, glyph.y);
            [
                (glyph.byte_offset, before),
                (glyph.byte_offset + glyph.parent.len_utf8(), after),
            ]
        });

    for (byte_offset, glyph_pos) in possible_positions {
        let dist = (pos_x - glyph_pos.x).abs();
        if dist < closest_dist {
            closest_byte_offset = byte_offset;
            closest_dist = dist;
        }
    }

    closest_byte_offset
}
