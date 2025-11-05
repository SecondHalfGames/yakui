use core::f32;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::mem;

use cosmic_text::Edit;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, FlexFit, Vec2};
use yakui_core::input::{KeyCode, Modifiers, MouseButton};
use yakui_core::navigation::NavDirection;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{context, Response};

use crate::clipboard::ClipboardHolder;
use crate::font::Fonts;
use crate::shapes;
use crate::style::{TextAlignment, TextStyle};
use crate::util::widget;
use crate::widgets::RenderTextResponse;
use crate::{auto_builders, colors, pad};

use super::{Pad, RenderText};

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct TextBox {
    pub style: TextStyle,
    pub min_width: f32,

    /// Drawn when no text has been set
    pub placeholder: Cow<'static, str>,
    pub placeholder_style: TextStyle,

    pub padding: Pad,
    pub fill: Option<Color>,
    pub radius: f32,

    pub selection_halo_color: Color,
    pub selected_bg_color: Color,
    pub cursor_color: Color,

    /// Whether or not enter triggers a loss of focus and if shift would be needed to override that
    pub inline_edit: bool,
    pub multiline: bool,
}

auto_builders!(TextBox {
    style: TextStyle,
    min_width: f32,

    placeholder: Cow<'static, str>,
    placeholder_style: TextStyle,

    padding: Pad,
    fill: Option<Color>,
    radius: f32,

    selection_halo_color: Color,
    selected_bg_color: Color,
    cursor_color: Color,

    inline_edit: bool,
    multiline: bool,
});

impl TextBox {
    pub fn new() -> Self {
        let style = TextStyle::label().align(TextAlignment::Start);
        let placeholder_style = style
            .clone()
            .color(style.color.lerp(&colors::BACKGROUND_3, 0.75));

        Self {
            style,
            min_width: 0.0,

            placeholder: String::new().into(),
            placeholder_style,

            padding: Pad::all(8.0),
            fill: Some(colors::BACKGROUND_3),
            radius: 6.0,

            selection_halo_color: Color::WHITE,
            selected_bg_color: Color::CORNFLOWER_BLUE.adjust(0.4),
            cursor_color: Color::RED,

            inline_edit: true,
            multiline: false,
        }
    }

    #[track_caller]
    pub fn show<'a, S: Into<&'a str>>(self, text: S) -> Response<TextBoxResponse> {
        widget::<TextBoxWidget>((self, text.into()))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DragState {
    None,
    DragStart,
    Dragging,
}

#[derive(Debug)]
pub struct TextBoxWidget {
    text: String,
    props: TextBox,

    /// Whether this widget is focused and receiving input from the user.
    active: bool,
    activated: bool,
    lost_focus: bool,
    drag: DragState,

    preedit_text: String,
    preedit_cursor: Option<cosmic_text::Cursor>,
    preedit_cursor_end: Option<cosmic_text::Cursor>,

    render_text_response: RenderTextResponse,

    cosmic_editor: RefCell<Option<cosmic_text::Editor<'static>>>,
    is_textbox_empty: bool,
    size_changed: Cell<bool>,
    scroll_changed: Cell<bool>,
    text_changed: Cell<bool>,
    /// Whether the Cosmic Text editor context has changed the text since the
    /// previous update. Edits from the user take precedence over edits from the
    /// application.
    text_changed_by_cosmic: Cell<bool>,
}

pub struct TextBoxResponse {
    /// If the contents of the textbox are different than what was passed into
    /// props, contains the new string.
    pub text: Option<String>,

    /// Whether or not the textbox is currently active (i.e. focused by the user).
    pub active: bool,

    /// Whether the user pressed "Enter" in this textbox. This only happens when
    /// the textbox is inline.
    pub activated: bool,

    /// Whether the textbox lost focus.
    pub lost_focus: bool,
}

impl Widget for TextBoxWidget {
    type Props<'a> = (TextBox, &'a str);
    type Response = TextBoxResponse;

    fn new() -> Self {
        Self {
            text: String::new(),
            props: TextBox::new(),

            active: false,
            activated: false,
            lost_focus: false,
            drag: DragState::None,

            preedit_text: String::new(),
            preedit_cursor: None,
            preedit_cursor_end: None,

            render_text_response: RenderTextResponse::default(),

            cosmic_editor: RefCell::new(None),
            is_textbox_empty: true,
            size_changed: Cell::new(true),
            scroll_changed: Cell::new(false),
            text_changed: Cell::new(true),
            text_changed_by_cosmic: Cell::new(false),
        }
    }

    fn flex(&self) -> (u32, FlexFit) {
        (1, FlexFit::Loose)
    }

    fn update(&mut self, (props, text): Self::Props<'_>) -> Self::Response {
        let text_changed_by_caller = text != self.text || props.style != self.props.style;
        if text_changed_by_caller {
            self.text_changed_by_cosmic.set(false);
        }
        let text_changed_by_cosmic = self.text_changed_by_cosmic.take();
        self.text_changed
            .set(text_changed_by_caller || text_changed_by_cosmic);

        self.props = props;
        if text != self.text {
            self.text = text.to_string();
        }

        let fonts = context::dom().get_global_or_init(Fonts::default);
        fonts.with_inner(|fonts| {
            if self.cosmic_editor.borrow().is_none() {
                self.cosmic_editor.replace(Some(cosmic_text::Editor::new(
                    cosmic_text::BufferRef::Owned(cosmic_text::Buffer::new(
                        &mut fonts.font_system,
                        // dummy value, updated during layout
                        // this is really just to make sure `editor` exists when we set the text a few lines later
                        cosmic_text::Metrics {
                            font_size: 1.0,
                            line_height: 1.0,
                        },
                    )),
                )));
            }
        });

        let mut editor_ref = self.cosmic_editor.borrow_mut();
        let editor = editor_ref.as_mut().unwrap();

        let size_changed = self.size_changed.take();
        let scroll_changed = self.scroll_changed.get();

        let mut new_scroll = None;

        fonts.with_inner(|fonts| {
            if text_changed_by_caller {
                editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                editor.set_selection(cosmic_text::Selection::None);
                editor.with_buffer_mut(|buffer| {
                    buffer.set_text(
                        &self.text,
                        &fonts
                            .font_selection
                            .get_cosmic_attrs(&self.props.style.font),
                        cosmic_text::Shaping::Advanced,
                        None,
                    );
                });
                editor.action(
                    &mut fonts.font_system,
                    cosmic_text::Action::Motion(cosmic_text::Motion::BufferEnd),
                );
            }

            if text_changed_by_caller || text_changed_by_cosmic {
                editor.with_buffer_mut(|buffer| {
                    // apply styles
                    for line in buffer.lines.iter_mut() {
                        line.set_align(self.props.style.align.into());
                    }

                    self.is_textbox_empty = !(buffer.lines.len() > 1
                        || buffer.lines.iter().any(|v| !v.text().is_empty()));
                });
            }

            if scroll_changed || size_changed {
                // TODO: Madeline Sparkles: this entire thing shouldn't be needed but there's a bug in cosmic-text.
                editor.with_buffer_mut(|buffer| {
                    let mut scroll = buffer.scroll();
                    let scrollable_area = self.render_text_response.measured_size.y
                        - self.render_text_response.physical_size.y;
                    scroll.vertical = scroll.vertical.clamp(0.0, scrollable_area.max(0.0));

                    if scroll != buffer.scroll() {
                        buffer.set_scroll(scroll);
                        self.scroll_changed.set(true);
                    }
                });
            }

            new_scroll = Some(editor.with_buffer(|buffer| buffer.scroll()));
        });

        if text_changed_by_cosmic {
            self.text = editor.with_buffer(|buffer| {
                buffer
                    .lines
                    .iter()
                    .map(|v| v.text())
                    .collect::<Vec<_>>()
                    .join("\n")
            });
        }

        pad(self.props.padding, || {
            let text = if self.is_textbox_empty {
                &self.props.placeholder
            } else {
                self.text.as_str()
            };

            let style = if self.is_textbox_empty {
                &self.props.placeholder_style
            } else {
                &self.props.style
            };

            let mut render_text = RenderText::new()
                .inline(true)
                .min_width(self.props.min_width);

            if self.active {
                let cursor = editor.cursor();
                let selection = editor.selection_bounds();

                render_text.cursor = Some(cursor);
                render_text.selection = selection;
                render_text.preedit_cursor = self.preedit_cursor;
                render_text.preedit_cursor_end = self.preedit_cursor_end;
                render_text.selected_bg_color = self.props.selected_bg_color;
                render_text.cursor_color = self.props.cursor_color;
            }

            let response = render_text
                .show_with_scroll(text, style, new_scroll)
                .into_inner();

            if self.render_text_response != response {
                self.size_changed.set(true);
            }
            self.render_text_response = response;
        });

        Self::Response {
            text: if text_changed_by_cosmic {
                Some(self.text.clone())
            } else {
                None
            },
            active: self.active,
            activated: mem::take(&mut self.activated),
            lost_focus: mem::take(&mut self.lost_focus),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.active {
            ctx.input.enable_text_input();
        }

        let size_changed = self.size_changed.get();

        let text_changed = self.text_changed.take();
        let scroll_changed = self.scroll_changed.take();

        let mut editor_ref = self.cosmic_editor.borrow_mut();
        let editor = editor_ref.as_mut().unwrap();

        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        fonts.with_inner(|fonts| {
            let scroll = editor.with_buffer(|buffer| buffer.scroll());

            if size_changed {
                editor.with_buffer_mut(|buffer| {
                    buffer.set_metrics(self.props.style.to_metrics(ctx.layout.scale_factor()));
                    buffer.set_size(
                        Some(self.render_text_response.physical_size.x),
                        self.render_text_response.max_size.1,
                    );
                });
            }

            if size_changed || text_changed || scroll_changed {
                editor.shape_as_needed(&mut fonts.font_system, text_changed);
            }

            let new_scroll = editor.with_buffer(|buffer| buffer.scroll());
            if scroll != new_scroll {
                self.scroll_changed.set(true);
            }
        });

        self.default_layout(ctx, constraints)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;

        if let Some(fill_color) = self.props.fill {
            let mut bg = shapes::RoundedRectangle::new(layout_rect, self.props.radius);
            bg.color = fill_color;
            bg.add(ctx.paint);
        }

        if self.active {
            shapes::selection_halo(ctx.paint, layout_rect, self.props.selection_halo_color);
        }

        self.default_paint(ctx);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
            | EventInterest::MOUSE_OUTSIDE
            | EventInterest::MOUSE_MOVE
            | EventInterest::FOCUS
            | EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let mut editor_ref = self.cosmic_editor.borrow_mut();
        let editor = editor_ref.as_mut().unwrap();

        match event {
            WidgetEvent::FocusChanged(focused) => {
                if *focused {
                    self.active = true;
                    ctx.input.set_selection(Some(ctx.dom.current()));
                } else {
                    self.active = false;
                    if ctx.input.selection() == Some(ctx.dom.current()) {
                        ctx.input.set_selection(None);
                    }
                }

                if !*focused {
                    self.lost_focus = true;
                    editor.set_selection(cosmic_text::Selection::None);
                }
                EventResponse::Sink
            }

            WidgetEvent::MouseScroll {
                delta,
                modifiers: _,
            } => {
                fonts.with_inner(|fonts| {
                    editor.action(
                        &mut fonts.font_system,
                        cosmic_text::Action::Scroll { pixels: delta.y },
                    );
                    self.scroll_changed.set(true);
                });

                EventResponse::Sink
            }

            WidgetEvent::MouseMoved(Some(position)) => {
                if self.drag == DragState::DragStart {
                    self.drag = DragState::Dragging;

                    EventResponse::Sink
                } else if self.drag == DragState::Dragging {
                    if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                        let scale_factor = ctx.layout.scale_factor();
                        let relative_pos =
                            position - (layout.rect.pos() + self.props.padding.offset());
                        let text_pos = (relative_pos * scale_factor).round().as_ivec2();

                        fonts.with_inner(|fonts| {
                            editor.action(
                                &mut fonts.font_system,
                                cosmic_text::Action::Drag {
                                    x: text_pos.x,
                                    y: text_pos.y,
                                },
                            );
                            self.scroll_changed.set(true);
                        });
                    }

                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }

            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                inside,
                down,
                position,
                modifiers,
                ..
            } => {
                if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                    let scale_factor = ctx.layout.scale_factor();
                    let relative_pos = position - (layout.rect.pos() + self.props.padding.offset());
                    let text_pos = (relative_pos * scale_factor).round().as_ivec2();

                    fonts.with_inner(|fonts| {
                        if *inside && *down {
                            if self.drag == DragState::None {
                                self.drag = DragState::DragStart;
                            }

                            if modifiers.shift() {
                                // TODO wait for cosmic text for shift clicking selection
                                // Madeline Sparkles: emulating this with a drag
                                editor.action(
                                    &mut fonts.font_system,
                                    cosmic_text::Action::Drag {
                                        x: text_pos.x,
                                        y: text_pos.y,
                                    },
                                );
                                self.scroll_changed.set(true);
                            } else {
                                editor.action(
                                    &mut fonts.font_system,
                                    cosmic_text::Action::Click {
                                        x: text_pos.x,
                                        y: text_pos.y,
                                    },
                                );
                                self.scroll_changed.set(true);
                            }
                        } else {
                            self.drag = DragState::None;
                        }
                    });
                }

                if *inside {
                    if *down {
                        self.active = true;
                        ctx.input.set_selection(Some(ctx.dom.current()));
                        EventResponse::Sink
                    } else {
                        EventResponse::Bubble
                    }
                } else {
                    self.active = false;
                    if ctx.input.selection() == Some(ctx.dom.current()) {
                        ctx.input.set_selection(None);
                    }
                    EventResponse::Bubble
                }
            }

            WidgetEvent::KeyChanged {
                key,
                down,
                modifiers,
                ..
            } => {
                if !down || self.preedit_cursor.is_some() {
                    return EventResponse::Bubble;
                }

                enum Select {
                    DeselectNoAffinity,
                    DeselectPrevAffinity,
                    DeselectNextAffinity,
                    CurrentCursor,
                }

                let mut select = None;
                let mut action = None;

                let (bound_start, bound_end) = editor.selection_bounds().unwrap_or_else(|| {
                    let cursor = editor.cursor();
                    (cursor, cursor)
                });

                match key {
                    KeyCode::ArrowLeft => {
                        // TODO: Madeline Sparkles: for all of these ctrl uses, I'm not sure how Mac users have it
                        // if anyone knows how text stuff like this works on Mac, please tell.
                        if main_modifier(modifiers) {
                            action = Some(cosmic_text::Action::Motion(
                                cosmic_text::Motion::PreviousWord,
                            ));
                            self.scroll_changed.set(true);
                        } else {
                            action =
                                Some(cosmic_text::Action::Motion(cosmic_text::Motion::Previous));
                            self.scroll_changed.set(true);
                        }

                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectPrevAffinity);
                        }
                    }
                    KeyCode::ArrowRight => {
                        if main_modifier(modifiers) {
                            action =
                                Some(cosmic_text::Action::Motion(cosmic_text::Motion::NextWord));
                            self.scroll_changed.set(true);
                        } else {
                            action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::Next));
                            self.scroll_changed.set(true);
                        }

                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectNextAffinity);
                        }
                    }
                    KeyCode::ArrowUp | KeyCode::PageUp => {
                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectPrevAffinity);
                        }

                        action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::Up));
                        self.scroll_changed.set(true);
                    }
                    KeyCode::ArrowDown | KeyCode::PageDown => {
                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectNextAffinity);
                        }

                        action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::Down));
                        self.scroll_changed.set(true);
                    }
                    KeyCode::Home => {
                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectNoAffinity);
                        }

                        action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::Home));
                        self.scroll_changed.set(true);
                    }
                    KeyCode::End => {
                        if modifiers.shift() {
                            select = Some(Select::CurrentCursor);
                        } else {
                            select = Some(Select::DeselectNoAffinity);
                        }

                        action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::End));
                        self.scroll_changed.set(true);
                    }
                    KeyCode::Tab => {
                        if modifiers.shift() {
                            ctx.input.navigate(NavDirection::Previous);
                        } else {
                            ctx.input.navigate(NavDirection::Next);
                        }
                    }
                    KeyCode::Escape => {
                        action = Some(cosmic_text::Action::Escape);
                        if self.props.inline_edit {
                            self.active = false;
                            if ctx.input.selection() == Some(ctx.dom.current()) {
                                ctx.input.set_selection(None);
                            }
                        }
                    }

                    KeyCode::Backspace => {
                        action = Some(cosmic_text::Action::Backspace);
                        self.text_changed_by_cosmic.set(true);
                    }
                    KeyCode::Delete => {
                        action = Some(cosmic_text::Action::Delete);
                        self.text_changed_by_cosmic.set(true);
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        if self.props.inline_edit {
                            if self.props.multiline && modifiers.shift() {
                                action = Some(cosmic_text::Action::Enter);
                                self.text_changed_by_cosmic.set(true);
                            } else {
                                self.activated = true;
                                self.active = false;
                                if ctx.input.selection() == Some(ctx.dom.current()) {
                                    ctx.input.set_selection(None);
                                }
                            }
                        } else {
                            action = Some(cosmic_text::Action::Enter);
                            self.text_changed_by_cosmic.set(true);
                        }
                    }

                    KeyCode::KeyA if main_modifier(modifiers) => {
                        fonts.with_inner(|fonts| {
                            editor.action(
                                &mut fonts.font_system,
                                cosmic_text::Action::Motion(cosmic_text::Motion::BufferStart),
                            );
                            editor.set_selection(cosmic_text::Selection::Normal(editor.cursor()));
                            editor.action(
                                &mut fonts.font_system,
                                cosmic_text::Action::Motion(cosmic_text::Motion::BufferEnd),
                            );
                            self.scroll_changed.set(true);
                        });
                    }
                    KeyCode::KeyX if main_modifier(modifiers) => {
                        let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                        if let Some(text) = editor.copy_selection() {
                            clipboard.copy(&text);
                        }
                        editor.delete_selection();
                        self.text_changed_by_cosmic.set(true);
                    }
                    KeyCode::KeyC if main_modifier(modifiers) => {
                        let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                        if let Some(text) = editor.copy_selection() {
                            clipboard.copy(&text);
                        }
                    }
                    KeyCode::KeyV if main_modifier(modifiers) => {
                        let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                        if let Some(text) = clipboard.paste() {
                            editor.insert_string(&text, None);
                            self.text_changed_by_cosmic.set(true);
                        }
                    }
                    _ => {}
                }

                if let Some(select) = select {
                    let cursor = editor.cursor();

                    match select {
                        Select::DeselectNoAffinity => {
                            editor.set_selection(cosmic_text::Selection::None);
                        }
                        Select::DeselectPrevAffinity => {
                            if bound_start != bound_end {
                                if let Some(cosmic_text::Action::Motion(
                                    cosmic_text::Motion::Previous,
                                )) = action
                                {
                                    action = None;

                                    fonts.with_inner(|fonts| {
                                        editor.action(
                                            &mut fonts.font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::BufferStart,
                                            ),
                                        );
                                        let buffer_start_cursor = editor.cursor();
                                        editor.action(
                                            &mut fonts.font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::PreviousWord,
                                            ),
                                        );
                                        let prev_word_cursor = editor.cursor();

                                        editor.set_cursor(cursor);

                                        if bound_start == buffer_start_cursor {
                                            editor.set_cursor(buffer_start_cursor);
                                        }

                                        if bound_start == prev_word_cursor {
                                            editor.set_cursor(prev_word_cursor);
                                        }
                                    });
                                }
                            }

                            editor.set_selection(cosmic_text::Selection::None);
                        }
                        Select::DeselectNextAffinity => {
                            if bound_start != bound_end {
                                if let Some(cosmic_text::Action::Motion(
                                    cosmic_text::Motion::Next,
                                )) = action
                                {
                                    action = None;

                                    fonts.with_inner(|fonts| {
                                        editor.action(
                                            &mut fonts.font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::BufferEnd,
                                            ),
                                        );
                                        let buffer_end_cursor = editor.cursor();
                                        editor.action(
                                            &mut fonts.font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::NextWord,
                                            ),
                                        );
                                        let next_word_cursor = editor.cursor();

                                        editor.set_cursor(cursor);

                                        if bound_end == buffer_end_cursor {
                                            editor.set_cursor(buffer_end_cursor);
                                        }

                                        if bound_end == next_word_cursor {
                                            editor.set_cursor(next_word_cursor);
                                        }
                                    });
                                }
                            }

                            editor.set_selection(cosmic_text::Selection::None);
                        }
                        Select::CurrentCursor => {
                            if bound_start == bound_end {
                                editor.set_selection(cosmic_text::Selection::Normal(cursor));
                            }
                        }
                    }
                }

                if let Some(action) = action {
                    fonts.with_inner(|fonts| {
                        editor.action(&mut fonts.font_system, action);
                    });
                }

                EventResponse::Sink
            }
            WidgetEvent::TextPreedit(text, position) => {
                if text.is_empty() && !self.preedit_text.is_empty() {
                    self.preedit_text = String::new();
                    let preedit_cursor = self.preedit_cursor.take().unwrap();

                    if let Some(preedit_cursor_end) = self.preedit_cursor_end.take() {
                        if preedit_cursor < preedit_cursor_end {
                            editor.delete_range(preedit_cursor, preedit_cursor_end);
                        }
                    }
                    editor.set_cursor(preedit_cursor);

                    self.text_changed_by_cosmic.set(true);
                } else if !text.is_empty() && *text != self.preedit_text {
                    if self.preedit_cursor.is_none() {
                        self.preedit_cursor = Some(editor.cursor());
                    }

                    let preedit_cursor = self.preedit_cursor.unwrap();

                    editor.set_selection(cosmic_text::Selection::None);
                    if let Some(preedit_cursor_end) = self.preedit_cursor_end {
                        if preedit_cursor < preedit_cursor_end {
                            editor.delete_range(preedit_cursor, preedit_cursor_end);
                            editor.set_cursor(preedit_cursor);
                        }
                    }
                    editor.insert_string(text, None);
                    self.preedit_cursor_end = Some(editor.cursor());

                    self.preedit_text = text.clone();

                    self.text_changed_by_cosmic.set(true);
                }

                if let Some(cursor) = self.preedit_cursor {
                    if let &Some((_start, end)) = position {
                        editor.set_cursor(cosmic_text::Cursor {
                            index: cursor.index + end,
                            ..cursor
                        });
                    }
                }

                EventResponse::Sink
            }
            WidgetEvent::TextInput(c, modifiers) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                if !modifiers.ctrl() && !modifiers.meta() {
                    fonts.with_inner(|fonts| {
                        editor.action(&mut fonts.font_system, cosmic_text::Action::Insert(*c));
                        self.text_changed_by_cosmic.set(true);
                    });
                }

                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}

/// Tells whether the set of modifiers contains the primary modifier, like ctrl
/// on Windows or Linux or Command on macOS.
fn main_modifier(modifiers: &Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.meta()
    } else {
        modifiers.ctrl()
    }
}
