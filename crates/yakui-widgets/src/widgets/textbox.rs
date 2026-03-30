use std::cell::{Cell, RefCell};
use std::mem;

use cosmic_text::{Edit, Selection};
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::input::{KeyCode, Modifiers, MouseButton};
use yakui_core::navigation::NavDirection;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::clipboard::ClipboardHolder;
use crate::font::Fonts;
use crate::shapes::{self, RoundedRectangle};
use crate::style::{TextAlignment, TextStyle};
use crate::util::widget;
use crate::{auto_builders, colors, pad};

use super::{Pad, RenderText};

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct TextBox {
    pub text: String,

    pub style: TextStyle,
    pub padding: Pad,
    pub fill: Option<Color>,
    pub radius: f32,

    /// Whether or not enter triggers a loss of focus and if shift would be needed to override that
    pub inline_edit: bool,
    pub multiline: bool,

    pub selection_halo_color: Color,
    pub selected_bg_color: Color,
    pub cursor_color: Color,

    /// Drawn when no text has been set
    pub placeholder: String,
}

auto_builders!(TextBox {
    style: TextStyle,
    padding: Pad,
    fill: Option<Color>,
    radius: f32,
    inline_edit: bool,
    multiline: bool,
    selection_halo_color: Color,
    selected_bg_color: Color,
    cursor_color: Color,
    placeholder: String,
});

impl TextBox {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),

            style: TextStyle::label().align(TextAlignment::Start),
            padding: Pad::all(8.0),
            fill: Some(colors::BACKGROUND_3),
            radius: 6.0,

            inline_edit: true,
            multiline: false,

            selection_halo_color: Color::WHITE,
            selected_bg_color: Color::CORNFLOWER_BLUE.adjust(0.4),
            cursor_color: Color::RED,

            placeholder: String::new(),
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<TextBoxResponse> {
        widget::<TextBoxWidget>(self)
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
    props: TextBox,

    /// Whether the caller of this widget has changed `props.text` since the
    /// previous update.
    text_changed_by_caller: bool,

    /// Whether the Cosmic Text editor context has changed the text since the
    /// previous update. Edits from the user take precedence over edits from the
    /// application.
    text_changed_by_cosmic: Cell<bool>,

    /// Whether this widget is focused and receiving input from the user.
    active: bool,

    activated: bool,
    lost_focus: bool,
    drag: DragState,
    cosmic_editor: RefCell<Option<cosmic_text::Editor<'static>>>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,
}

pub struct TextBoxResponse {
    /// If the contents of the textbox are different than what was passed into
    /// props, contains the new string.
    pub text: Option<String>,

    /// Whether the user pressed "Enter" in this textbox. This only happens when
    /// the textbox is inline.
    pub activated: bool,

    /// Whether the textbox lost focus.
    pub lost_focus: bool,
}

impl Widget for TextBoxWidget {
    type Props<'a> = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        Self {
            props: TextBox::new(String::new()),
            text_changed_by_caller: false,
            active: false,
            activated: false,
            lost_focus: false,
            drag: DragState::None,
            cosmic_editor: RefCell::new(None),
            max_size: Cell::default(),
            text_changed_by_cosmic: Cell::default(),
            scale_factor: Cell::default(),
        }
    }

    fn update(&mut self, mut props: Self::Props<'_>) -> Self::Response {
        if self.text_changed_by_cosmic.get() {
            self.text_changed_by_caller = false;
            props.text = mem::take(&mut self.props.text);
        } else {
            self.text_changed_by_caller = props.text != self.props.text;
        }

        self.props = props;

        let mut style = self.props.style.clone();
        let mut scroll = None;

        let mut is_empty = false;

        let editor_text = self
            .cosmic_editor
            .borrow()
            .as_ref()
            .map(|editor| {
                editor.with_buffer(|buffer| {
                    scroll = Some(buffer.scroll());
                    is_empty = buffer.lines.iter().all(|v| v.text().is_empty());

                    buffer
                        .lines
                        .iter()
                        .map(|v| v.text())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            })
            .unwrap_or_default();

        if is_empty {
            // Dim towards background
            style.color = style
                .color
                .lerp(&self.props.fill.unwrap_or(Color::CLEAR), 0.75);
        }

        pad(self.props.padding, || {
            let render_text = if is_empty {
                self.props.placeholder.clone()
            } else if self.text_changed_by_cosmic.get() {
                editor_text.clone()
            } else {
                self.props.text.clone()
            };

            RenderText::with_style(render_text, style).show_with_scroll(scroll);
        });

        if self.text_changed_by_cosmic.get() {
            self.props.text = editor_text.clone();
        }

        Self::Response {
            text: if self.text_changed_by_cosmic.take() {
                Some(editor_text)
            } else {
                None
            },
            activated: mem::take(&mut self.activated),
            lost_focus: mem::take(&mut self.lost_focus),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.active {
            ctx.input.enable_text_input();
        }

        let max_width = constraints.max.x.is_finite().then_some(
            (constraints.max.x - self.props.padding.offset().x * 2.0) * ctx.layout.scale_factor(),
        );
        let max_height = constraints.max.y.is_finite().then_some(
            (constraints.max.y - self.props.padding.offset().y * 2.0) * ctx.layout.scale_factor(),
        );
        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        fonts.with_system(|font_system| {
            if self.cosmic_editor.borrow().is_none() {
                self.cosmic_editor.replace(Some(cosmic_text::Editor::new(
                    cosmic_text::BufferRef::Owned(cosmic_text::Buffer::new(
                        font_system,
                        self.props.style.to_metrics(ctx.layout.scale_factor()),
                    )),
                )));
            }

            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                if self.scale_factor.get() != Some(ctx.layout.scale_factor())
                    || self.max_size.get() != Some(max_size)
                {
                    editor.with_buffer_mut(|buffer| {
                        buffer.set_metrics(
                            font_system,
                            self.props.style.to_metrics(ctx.layout.scale_factor()),
                        );

                        buffer.set_size(font_system, max_width, max_height);
                    });

                    self.scale_factor.set(Some(ctx.layout.scale_factor()));
                    self.max_size.replace(Some(max_size));
                }

                if self.text_changed_by_caller {
                    editor.with_buffer_mut(|buffer| {
                        buffer.set_text(
                            font_system,
                            &self.props.text,
                            &self.props.style.attrs.as_attrs(),
                            cosmic_text::Shaping::Advanced,
                            None,
                        );
                    });

                    editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                }

                // Perf note: https://github.com/pop-os/cosmic-text/issues/166
                editor.with_buffer_mut(|buffer| {
                    for buffer_line in buffer.lines.iter_mut() {
                        buffer_line.set_align(Some(self.props.style.align.into()));
                    }
                    buffer.shape_until_scroll(font_system, true);
                });
            }
        });

        self.default_layout(ctx, constraints)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        fonts.with_system(|font_system| {
            if let Some(fill_color) = self.props.fill {
                let mut bg = RoundedRectangle::new(layout_node.rect, self.props.radius);
                bg.color = fill_color;
                bg.add(ctx.paint);
            }

            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                editor.shape_as_needed(font_system, false);

                let cursor = editor.cursor();
                let selection = editor.selection_bounds();
                editor.with_buffer_mut(|buffer| {
                    let inv_scale_factor = 1.0 / ctx.layout.scale_factor();

                    if let Some((a, b)) = selection {
                        for ((x, y), (w, h)) in buffer
                            .layout_runs()
                            .filter_map(|layout| {
                                let (x, w) = layout.highlight(a, b)?;
                                let (y, h) = (layout.line_top, layout.line_height);

                                Some(((x, y), (w, h)))
                            })
                            .filter(|(_, (w, _))| *w > 0.1)
                        {
                            let mut bg = PaintRect::new(Rect::from_pos_size(
                                layout_node.rect.pos()
                                    + self.props.padding.offset()
                                    + Vec2::new(x, y) * inv_scale_factor,
                                Vec2::new(w, h) * inv_scale_factor,
                            ));
                            bg.color = self.props.selected_bg_color;
                            bg.add(ctx.paint);
                        }
                    }

                    if self.active {
                        let ((x, y), (_, h)) = buffer
                            .layout_runs()
                            .find_map(|layout| {
                                let (x, w) = layout.highlight(cursor, cursor)?;
                                let (y, h) = (layout.line_top, layout.line_height);

                                Some(((x, y), (w, h)))
                            })
                            .unwrap_or(((0.0, 0.0), (0.0, buffer.metrics().line_height)));

                        let mut bg = PaintRect::new(Rect::from_pos_size(
                            layout_node.rect.pos()
                                + self.props.padding.offset()
                                + Vec2::new(x, y) * inv_scale_factor,
                            Vec2::new(1.5, h) * inv_scale_factor,
                        ));
                        bg.color = self.props.cursor_color;
                        bg.add(ctx.paint);
                    }
                });
            }
        });

        if self.active {
            shapes::selection_halo(ctx.paint, layout_node.rect, self.props.selection_halo_color);
        }

        self.default_paint(ctx);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
            | EventInterest::FOCUS
            | EventInterest::FOCUSED_KEYBOARD
            | EventInterest::MOUSE_MOVE
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::FocusChanged(focused) => {
                self.active = *focused;
                if !*focused {
                    self.lost_focus = true;
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                    }
                }
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
                            *position - layout.rect.pos() - self.props.padding.offset();
                        let glyph_pos = (relative_pos * scale_factor).round().as_ivec2();

                        let fonts = ctx.dom.get_global_or_init(Fonts::default);
                        fonts.with_system(|font_system| {
                            if let Some(editor) = self.cosmic_editor.get_mut() {
                                editor.action(
                                    font_system,
                                    cosmic_text::Action::Drag {
                                        x: glyph_pos.x,
                                        y: glyph_pos.y,
                                    },
                                );
                            }
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
                if !inside {
                    return EventResponse::Sink;
                }

                if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                    let scale_factor = ctx.layout.scale_factor();
                    let relative_pos = *position - layout.rect.pos() - self.props.padding.offset();
                    let glyph_pos = (relative_pos * scale_factor).round().as_ivec2();

                    let fonts = ctx.dom.get_global_or_init(Fonts::default);
                    fonts.with_system(|font_system| {
                        if *down {
                            if self.drag == DragState::None {
                                self.drag = DragState::DragStart;
                            }

                            if let Some(editor) = self.cosmic_editor.get_mut() {
                                if modifiers.shift() {
                                    // TODO wait for cosmic text for shift clicking selection
                                    // Madeline Sparkles: emulating this with a drag
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Drag {
                                            x: glyph_pos.x,
                                            y: glyph_pos.y,
                                        },
                                    );
                                } else {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Click {
                                            x: glyph_pos.x,
                                            y: glyph_pos.y,
                                        },
                                    );
                                }
                            }
                        } else {
                            self.drag = DragState::None;
                        }
                    });
                }

                ctx.input.set_selection(Some(ctx.dom.current()));

                EventResponse::Sink
            }

            WidgetEvent::KeyChanged {
                key,
                down,
                modifiers,
                ..
            } => {
                let fonts = ctx.dom.get_global_or_init(Fonts::default);
                fonts.with_system(|font_system| {
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        enum SelectMove {
                            Deselect,
                            Left,
                            Right,
                        }

                        let mut select_move = None;
                        let original_bounds = editor.selection_bounds().unwrap_or_else(|| {
                            let cursor = editor.cursor();
                            (cursor, cursor)
                        });
                        let res;

                        match key {
                            KeyCode::Tab => {
                                if *down {
                                    if modifiers.shift() {
                                        ctx.input.navigate(NavDirection::Previous);
                                    } else {
                                        ctx.input.navigate(NavDirection::Next);
                                    }
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::ArrowLeft => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Left);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    if modifiers.ctrl() {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::LeftWord,
                                            ),
                                        );
                                    } else {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(cosmic_text::Motion::Left),
                                        );
                                    }
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::ArrowRight => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Right);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    if modifiers.ctrl() {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::RightWord,
                                            ),
                                        );
                                    } else {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(cosmic_text::Motion::Right),
                                        );
                                    }
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::ArrowUp => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Left);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Up),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::ArrowDown => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Right);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Down),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::PageUp => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Left);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::PageUp),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::PageDown => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Right);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::PageDown),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::Backspace => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Backspace);
                                    self.text_changed_by_cosmic.set(true);
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::Delete => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Delete);
                                    self.text_changed_by_cosmic.set(true);
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::Home => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Left);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Home),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::End => {
                                if *down {
                                    if modifiers.shift() {
                                        select_move = Some(SelectMove::Right);
                                    } else {
                                        select_move = Some(SelectMove::Deselect);
                                    }

                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::End),
                                    );
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::Enter | KeyCode::NumpadEnter => {
                                if *down {
                                    if self.props.inline_edit {
                                        if self.props.multiline && modifiers.shift() {
                                            editor.action(font_system, cosmic_text::Action::Enter);
                                            self.text_changed_by_cosmic.set(true);
                                        } else {
                                            self.activated = true;
                                            ctx.input.set_selection(None);
                                        }
                                    } else {
                                        editor.action(font_system, cosmic_text::Action::Enter);
                                        self.text_changed_by_cosmic.set(true);
                                    }
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::Escape => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Escape);
                                    if self.props.inline_edit {
                                        ctx.input.set_selection(None);
                                    }
                                }
                                res = EventResponse::Sink;
                            }

                            KeyCode::KeyA if *down && main_modifier(modifiers) => {
                                editor.set_selection(cosmic_text::Selection::Line(editor.cursor()));

                                if let Some((_start, end)) = editor.selection_bounds() {
                                    editor.set_cursor(end);
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::KeyX if *down && main_modifier(modifiers) => {
                                let clipboard =
                                    ctx.dom.get_global_or_init(ClipboardHolder::default);

                                if let Some(text) = editor.copy_selection() {
                                    clipboard.copy(&text);
                                }
                                editor.delete_selection();
                                self.text_changed_by_cosmic.set(true);

                                res = EventResponse::Sink;
                            }

                            KeyCode::KeyC if *down && main_modifier(modifiers) => {
                                let clipboard =
                                    ctx.dom.get_global_or_init(ClipboardHolder::default);

                                if let Some(text) = editor.copy_selection() {
                                    clipboard.copy(&text);
                                }

                                res = EventResponse::Sink;
                            }

                            KeyCode::KeyV if *down && main_modifier(modifiers) => {
                                let clipboard =
                                    ctx.dom.get_global_or_init(ClipboardHolder::default);

                                if let Some(text) = clipboard.paste() {
                                    editor.insert_string(&text, None);
                                    self.text_changed_by_cosmic.set(true);
                                }

                                res = EventResponse::Sink;
                            }

                            _ => res = EventResponse::Sink,
                        }

                        match select_move {
                            Some(SelectMove::Deselect) => {
                                editor.set_selection(Selection::None);
                            }

                            Some(SelectMove::Left) => {
                                editor.set_selection(Selection::Normal(original_bounds.1));
                            }

                            Some(SelectMove::Right) => {
                                editor.set_selection(Selection::Normal(original_bounds.0));
                            }

                            None => {}
                        }

                        res
                    } else {
                        EventResponse::Bubble
                    }
                })
            }
            WidgetEvent::TextInput(c, modifiers) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                if !modifiers.ctrl() && !modifiers.meta() {
                    let fonts = ctx.dom.get_global_or_init(Fonts::default);
                    fonts.with_system(|font_system| {
                        if let Some(editor) = self.cosmic_editor.get_mut() {
                            editor.action(font_system, cosmic_text::Action::Insert(*c));
                            self.text_changed_by_cosmic.set(true);
                        }
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
