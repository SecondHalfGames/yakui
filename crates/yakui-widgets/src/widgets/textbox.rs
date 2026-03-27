use core::f32;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::mem;

use cosmic_text::Edit;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, FlexFit, Rect, Vec2};
use yakui_core::input::{KeyCode, Modifiers, MouseButton};
use yakui_core::navigation::NavDirection;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{context, Response};

use crate::clipboard::ClipboardHolder;
use crate::font::Fonts;
use crate::shapes::{self, RoundedRectangle};
use crate::style::{TextAlignment, TextStyle};
use crate::util::widget;
use crate::widgets::measure_text_width;
use crate::{auto_builders, colors, pad};

use super::{Pad, RenderText};

/// Code copied from cosmic-text that wasn't marked `pub`.
mod cosmic_text_util {
    use core::cmp;

    use cosmic_text::{Buffer, Cursor, LayoutRun};
    use unicode_segmentation::UnicodeSegmentation;

    /// See issue https://github.com/pop-os/cosmic-text/issues/437
    pub fn highlight<'a>(
        buffer: &'a Buffer,
        selection_bounds: Option<(Cursor, Cursor)>,
    ) -> impl Iterator<Item = ((f32, f32), (f32, f32))> + 'a {
        buffer.layout_runs().flat_map(move |run| {
            let line_i = run.line_i;
            let line_top = run.line_top;
            let line_height = run.line_height;

            if let Some((start, end)) = selection_bounds {
                if line_i >= start.line && line_i <= end.line {
                    let mut range_opt = None;
                    for glyph in run.glyphs {
                        // Guess x offset based on characters
                        let cluster = &run.text[glyph.start..glyph.end];
                        let total = cluster.grapheme_indices(true).count();
                        let mut c_x = glyph.x;
                        let c_w = glyph.w / total as f32;
                        for (i, c) in cluster.grapheme_indices(true) {
                            let c_start = glyph.start + i;
                            let c_end = glyph.start + i + c.len();
                            if (start.line != line_i || c_end > start.index)
                                && (end.line != line_i || c_start < end.index)
                            {
                                range_opt = match range_opt.take() {
                                    Some((min, max)) => Some((
                                        cmp::min(min, c_x as i32),
                                        cmp::max(max, (c_x + c_w) as i32),
                                    )),
                                    None => Some((c_x as i32, (c_x + c_w) as i32)),
                                };
                            } else if let Some((min, max)) = range_opt.take() {
                                return Some((
                                    (min as f32, line_top),
                                    (cmp::max(0, max - min) as f32, line_height),
                                ));
                            }
                            c_x += c_w;
                        }
                    }

                    if run.glyphs.is_empty() && end.line > line_i {
                        // Highlight all of internal empty lines
                        range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                    }

                    if let Some((mut min, mut max)) = range_opt.take() {
                        if end.line > line_i {
                            // Draw to end of line
                            if run.rtl {
                                min = 0;
                            } else {
                                max = buffer.size().0.unwrap_or(0.0) as i32;
                            }
                        }
                        return Some((
                            (min as f32, line_top),
                            (cmp::max(0, max - min) as f32, line_height),
                        ));
                    }
                }
            }

            None
        })
    }

    fn cursor_glyph_opt(cursor: &Cursor, run: &LayoutRun) -> Option<(usize, f32)> {
        if cursor.line == run.line_i {
            for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
                if cursor.index == glyph.start {
                    return Some((glyph_i, 0.0));
                } else if cursor.index > glyph.start && cursor.index < glyph.end {
                    // Guess x offset based on characters
                    let mut before = 0;
                    let mut total = 0;

                    let cluster = &run.text[glyph.start..glyph.end];
                    for (i, _) in cluster.grapheme_indices(true) {
                        if glyph.start + i < cursor.index {
                            before += 1;
                        }
                        total += 1;
                    }

                    let offset = glyph.w * (before as f32) / (total as f32);
                    return Some((glyph_i, offset));
                }
            }
            match run.glyphs.last() {
                Some(glyph) => {
                    if cursor.index == glyph.end {
                        return Some((run.glyphs.len(), 0.0));
                    }
                }
                None => {
                    return Some((0, 0.0));
                }
            }
        }
        None
    }

    /// See issue https://github.com/pop-os/cosmic-text/issues/361
    pub fn cursor_position(cursor: &Cursor, run: &LayoutRun) -> Option<(i32, i32)> {
        let (cursor_glyph, cursor_glyph_offset) = cursor_glyph_opt(cursor, run)?;
        let x = run.glyphs.get(cursor_glyph).map_or_else(
            || {
                run.glyphs.last().map_or(0, |glyph| {
                    if glyph.level.is_rtl() {
                        glyph.x as i32
                    } else {
                        (glyph.x + glyph.w) as i32
                    }
                })
            },
            |glyph| {
                if glyph.level.is_rtl() {
                    (glyph.x + glyph.w - cursor_glyph_offset) as i32
                } else {
                    (glyph.x + cursor_glyph_offset) as i32
                }
            },
        );

        Some((x, run.line_top as i32))
    }
}

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

    max_text_width: Cell<f32>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,

    cosmic_editor: RefCell<Option<cosmic_text::Editor<'static>>>,
    scroll_changed: Cell<bool>,
    text_changed: Cell<bool>,
    /// Whether the Cosmic Text editor context has changed the text since the
    /// previous update. Edits from the user take precedence over edits from the
    /// application.
    text_changed_by_cosmic: Cell<bool>,

    text_cursor: Cell<Option<Rect>>,
    can_scroll_down: Cell<bool>,
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

            max_text_width: Cell::default(),
            max_size: Cell::default(),
            scale_factor: Cell::default(),

            cosmic_editor: RefCell::new(None),
            scroll_changed: Cell::new(true),
            text_changed: Cell::new(true),
            text_changed_by_cosmic: Cell::new(true),

            text_cursor: Cell::default(),
            can_scroll_down: Cell::default(),
        }
    }

    fn flex(&self) -> (u32, FlexFit) {
        (1, FlexFit::Loose)
    }

    fn update(&mut self, (props, text): Self::Props<'_>) -> Self::Response {
        let text_changed_by_caller = text != self.text || props.style != self.props.style;
        if text_changed_by_caller {
            self.text_changed_by_cosmic.set(false);
            self.text = text.to_string();
        }
        let text_changed_by_cosmic = self.text_changed_by_cosmic.take();
        self.text_changed
            .set(text_changed_by_caller || text_changed_by_cosmic);

        self.props = props;

        let scroll_changed = self.scroll_changed.get();

        let mut scroll = None;
        let mut cursor = None;
        let mut is_textbox_empty = false;

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

            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                if text_changed_by_caller {
                    editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                    editor.set_selection(cosmic_text::Selection::None);
                    editor.with_buffer_mut(|buffer| {
                        buffer.set_text(
                            &mut fonts.font_system,
                            &self.text,
                            &fonts
                                .font_selection
                                .get_cosmic_attrs(&self.props.style.font),
                            cosmic_text::Shaping::Advanced,
                            self.props.style.align.into(),
                        );
                    });
                    editor.action(
                        &mut fonts.font_system,
                        cosmic_text::Action::Motion(cosmic_text::Motion::BufferEnd),
                    );
                }

                if text_changed_by_caller || text_changed_by_cosmic {
                    editor.with_buffer_mut(|buffer| {
                        for line in buffer.lines.iter_mut() {
                            line.set_align(self.props.style.align.into());
                        }
                    });
                }

                if text_changed_by_caller || text_changed_by_cosmic || scroll_changed {
                    editor.shape_as_needed(&mut fonts.font_system, true);
                }

                editor.with_buffer_mut(|buffer| {
                    // TODO: Madeline Sparkles: this entire thing shouldn't be needed but there's a bug in cosmic-text.
                    if text_changed_by_caller || text_changed_by_cosmic || scroll_changed {
                        let scroll = buffer.scroll();
                        let (max_width, max_height) = buffer.size();
                        buffer.set_size(&mut fonts.font_system, max_width, None);
                        let size_y = buffer
                            .layout_runs()
                            .map(|run| run.line_height)
                            .sum::<f32>()
                            .ceil();
                        buffer.set_size(&mut fonts.font_system, max_width, max_height);
                        buffer.set_scroll(scroll);

                        if Some(size_y) > max_height {
                            self.can_scroll_down.set(true);
                        } else {
                            self.can_scroll_down.set(false);
                        }
                    }

                    scroll = Some(buffer.scroll());
                    is_textbox_empty = !(buffer.lines.len() > 1
                        || buffer.lines.iter().any(|v| !v.text().is_empty()));
                });
                cursor = Some(editor.cursor());
            }
        });

        if text_changed_by_cosmic {
            self.text = self
                .cosmic_editor
                .borrow()
                .as_ref()
                .map(|editor| {
                    editor.with_buffer(|buffer| {
                        buffer
                            .lines
                            .iter()
                            .map(|v| v.text())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                })
                .unwrap_or_default();
        }

        pad(self.props.padding, || {
            let render_text = if is_textbox_empty {
                self.props.placeholder.clone()
            } else {
                self.text.clone().into()
            };

            let style = if is_textbox_empty {
                self.props.placeholder_style.clone()
            } else {
                self.props.style.clone()
            };

            RenderText::with_style(render_text, style)
                .inline(true)
                .min_width(self.props.min_width)
                .show_with_scroll(scroll, cursor);
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

        if let Some(cursor) = self.text_cursor.get() {
            ctx.input.set_text_cursor(cursor);
        }

        let max_width = (constraints.min.x.max(constraints.max.x)).max(self.props.min_width);
        let max_width = if max_width.is_normal() {
            let max_width = (max_width - self.props.padding.offset().x * 2.0).max(0.0);
            if max_width.is_normal() {
                Some((max_width * ctx.layout.scale_factor()).ceil())
            } else {
                None
            }
        } else {
            let max_width = (self.props.min_width - self.props.padding.offset().x * 2.0).max(0.0);
            if max_width.is_normal() {
                Some((max_width * ctx.layout.scale_factor()).ceil())
            } else {
                None
            }
        };

        let max_height = constraints.min.y.max(constraints.max.y);
        let max_height = if max_height.is_normal() {
            let max_height = (max_height - self.props.padding.offset().y * 2.0).max(0.0);
            if max_height.is_normal() {
                Some((max_height * ctx.layout.scale_factor()).ceil())
            } else {
                None
            }
        } else {
            None
        };

        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        fonts.with_inner(|fonts| {
            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                let text_changed = self.text_changed.take();
                let _scroll_changed = self.scroll_changed.take();
                let size_changed = self.max_size.get() != Some(max_size)
                    || self.scale_factor.get() != Some(ctx.layout.scale_factor());

                self.max_size.replace(Some(max_size));
                self.scale_factor.set(Some(ctx.layout.scale_factor()));

                if text_changed || size_changed {
                    self.max_text_width.set(
                        measure_text_width(
                            &mut fonts.font_system,
                            &mut fonts.font_selection,
                            &self.text,
                            &self.props.style,
                            ctx.layout.scale_factor(),
                            max_width,
                        )
                        .max(self.props.min_width * ctx.layout.scale_factor())
                        .ceil(),
                    );

                    editor.with_buffer_mut(|buffer| {
                        buffer.set_metrics(
                            &mut fonts.font_system,
                            self.props.style.to_metrics(ctx.layout.scale_factor()),
                        );
                        buffer.set_size(
                            &mut fonts.font_system,
                            Some(self.max_text_width.get()),
                            max_height,
                        );
                    });
                }
            }
        });

        self.default_layout(ctx, constraints)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        const CURSOR_WIDTH: f32 = 1.5;

        let inv_scale_factor = 1.0 / ctx.layout.scale_factor();

        let layout_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let offset = self.props.padding.offset();

        let text_rect = Rect::from_pos_size(
            layout_rect.pos()
                + offset
                + self.get_offset(ctx.layout.scale_factor()) * inv_scale_factor,
            layout_rect.size() - offset * 2.0,
        );

        let mut cursor_bound = text_rect;
        cursor_bound.set_size(Vec2::new(
            cursor_bound.size().x + cursor_bound.pos().x + CURSOR_WIDTH,
            cursor_bound.size().y,
        ));
        cursor_bound.set_pos(Vec2::new(0.0, cursor_bound.pos().y));

        if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
            let cursor = editor.cursor();
            let selection = editor.selection_bounds();

            if let Some(fill_color) = self.props.fill {
                let mut bg = RoundedRectangle::new(layout_rect, self.props.radius);
                bg.color = fill_color;
                bg.add(ctx.paint);
            }

            editor.with_buffer_mut(|buffer| {
                for ((x, y), (w, h)) in cosmic_text_util::highlight(buffer, selection) {
                    let pos = Vec2::new(x, y) * inv_scale_factor;
                    let size = Vec2::new(w, h) * inv_scale_factor;

                    let mut selection_rect = PaintRect::new(
                        cursor_bound.constrain(Rect::from_pos_size(text_rect.pos() + pos, size)),
                    );
                    selection_rect.color = self.props.selected_bg_color;
                    selection_rect.add(ctx.paint);
                }
            });

            if self.active {
                editor.with_buffer_mut(|buffer| {
                    let cursor_rect = buffer.layout_runs().find_map(|run| {
                        let (x, y) = cosmic_text_util::cursor_position(&cursor, &run)?;
                        let (x, y) = (x as f32, y as f32);
                        let (w, h) = (run.line_w, run.line_height);

                        Some(((x, y), (w, h)))
                    });

                    self.text_cursor.set(cursor_rect.map(|((x, y), (w, h))| {
                        Rect::from_pos_size(
                            text_rect.pos() * ctx.layout.scale_factor() + Vec2::new(x, y),
                            Vec2::new(w - x, h),
                        )
                    }));

                    if let Some(((x, y), (_, h))) = cursor_rect {
                        let pos =
                            Vec2::new(x * inv_scale_factor - CURSOR_WIDTH, y * inv_scale_factor);
                        let size = Vec2::new(CURSOR_WIDTH, h * inv_scale_factor);

                        let mut cursor_highlight = PaintRect::new(
                            cursor_bound
                                .constrain(Rect::from_pos_size(text_rect.pos() + pos, size)),
                        );
                        cursor_highlight.color = self.props.cursor_color;
                        cursor_highlight.add(ctx.paint);
                    }

                    if let Some((preedit_cursor, preedit_cursor_end)) =
                        self.preedit_cursor.zip(self.preedit_cursor_end)
                    {
                        for ((x, y), (w, h)) in cosmic_text_util::highlight(
                            buffer,
                            Some((preedit_cursor, preedit_cursor_end)),
                        ) {
                            let pos = Vec2::new(x, y + h) * inv_scale_factor - Vec2::new(0.0, 3.0);
                            let size = Vec2::new(w * inv_scale_factor, 3.0);

                            let mut preedit_highlight = PaintRect::new(
                                cursor_bound
                                    .constrain(Rect::from_pos_size(text_rect.pos() + pos, size)),
                            );
                            preedit_highlight.color = self.props.style.color;
                            preedit_highlight.add(ctx.paint);
                        }
                    }
                });
            }
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
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        editor.set_selection(cosmic_text::Selection::None);
                    }
                }
                EventResponse::Sink
            }

            WidgetEvent::MouseScroll {
                delta,
                modifiers: _,
            } => {
                if delta.y > 0.0 && !self.can_scroll_down.get() {
                    return EventResponse::Bubble;
                }

                if let Some(editor) = self.cosmic_editor.get_mut() {
                    fonts.with_inner(|fonts| {
                        editor.action(
                            &mut fonts.font_system,
                            cosmic_text::Action::Scroll { pixels: delta.y },
                        );
                        self.scroll_changed.set(true);
                    });
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
                        let text_pos = (relative_pos * scale_factor
                            - self.get_offset(ctx.layout.scale_factor()))
                        .round()
                        .as_ivec2();

                        fonts.with_inner(|fonts| {
                            if let Some(editor) = self.cosmic_editor.get_mut() {
                                editor.action(
                                    &mut fonts.font_system,
                                    cosmic_text::Action::Drag {
                                        x: text_pos.x,
                                        y: text_pos.y,
                                    },
                                );
                                self.scroll_changed.set(true);
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
                if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                    let scale_factor = ctx.layout.scale_factor();
                    let relative_pos = *position - layout.rect.pos() - self.props.padding.offset();
                    let text_pos = (relative_pos * scale_factor
                        - self.get_offset(ctx.layout.scale_factor()))
                    .round()
                    .as_ivec2();

                    fonts.with_inner(|fonts| {
                        if *inside && *down {
                            if self.drag == DragState::None {
                                self.drag = DragState::DragStart;
                            }

                            if let Some(editor) = self.cosmic_editor.get_mut() {
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
                if self.preedit_cursor.is_some() {
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

                let mut res = EventResponse::Bubble;

                if let Some(editor) = self.cosmic_editor.get_mut() {
                    let (bound_start, bound_end) = editor.selection_bounds().unwrap_or_else(|| {
                        let cursor = editor.cursor();
                        (cursor, cursor)
                    });

                    match key {
                        KeyCode::ArrowLeft => {
                            if *down {
                                // TODO: Madeline Sparkles: for all of these ctrl uses, I'm not sure how Mac users have it
                                // if anyone knows how text stuff like this works on Mac, please tell.
                                if modifiers.ctrl() {
                                    action = Some(cosmic_text::Action::Motion(
                                        cosmic_text::Motion::PreviousWord,
                                    ));
                                    self.scroll_changed.set(true);
                                } else {
                                    action = Some(cosmic_text::Action::Motion(
                                        cosmic_text::Motion::Previous,
                                    ));
                                    self.scroll_changed.set(true);
                                }

                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectPrevAffinity);
                                }
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::ArrowRight => {
                            if *down {
                                if modifiers.ctrl() {
                                    action = Some(cosmic_text::Action::Motion(
                                        cosmic_text::Motion::NextWord,
                                    ));
                                    self.scroll_changed.set(true);
                                } else {
                                    action = Some(cosmic_text::Action::Motion(
                                        cosmic_text::Motion::Next,
                                    ));
                                    self.scroll_changed.set(true);
                                }

                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectNextAffinity);
                                }
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::ArrowUp | KeyCode::PageUp => {
                            if *down {
                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectPrevAffinity);
                                }

                                action = Some(cosmic_text::Action::Motion(cosmic_text::Motion::Up));
                                self.scroll_changed.set(true);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::ArrowDown | KeyCode::PageDown => {
                            if *down {
                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectNextAffinity);
                                }

                                action =
                                    Some(cosmic_text::Action::Motion(cosmic_text::Motion::Down));
                                self.scroll_changed.set(true);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::Home => {
                            if *down {
                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectNoAffinity);
                                }

                                action =
                                    Some(cosmic_text::Action::Motion(cosmic_text::Motion::Home));
                                self.scroll_changed.set(true);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::End => {
                            if *down {
                                if modifiers.shift() {
                                    select = Some(Select::CurrentCursor);
                                } else {
                                    select = Some(Select::DeselectNoAffinity);
                                }

                                action =
                                    Some(cosmic_text::Action::Motion(cosmic_text::Motion::End));
                                self.scroll_changed.set(true);
                            }

                            res = EventResponse::Sink;
                        }
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
                        KeyCode::Escape => {
                            if *down {
                                action = Some(cosmic_text::Action::Escape);
                                if self.props.inline_edit {
                                    self.active = false;
                                    if ctx.input.selection() == Some(ctx.dom.current()) {
                                        ctx.input.set_selection(None);
                                    }
                                }
                            }
                            res = EventResponse::Sink;
                        }

                        KeyCode::Backspace => {
                            if *down {
                                action = Some(cosmic_text::Action::Backspace);
                                self.text_changed_by_cosmic.set(true);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::Delete => {
                            if *down {
                                action = Some(cosmic_text::Action::Delete);
                                self.text_changed_by_cosmic.set(true);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::Enter | KeyCode::NumpadEnter => {
                            if *down {
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

                            res = EventResponse::Sink;
                        }

                        KeyCode::KeyA if *down && main_modifier(modifiers) => {
                            fonts.with_inner(|fonts| {
                                editor.action(
                                    &mut fonts.font_system,
                                    cosmic_text::Action::Motion(cosmic_text::Motion::BufferStart),
                                );
                                editor
                                    .set_selection(cosmic_text::Selection::Normal(editor.cursor()));
                                editor.action(
                                    &mut fonts.font_system,
                                    cosmic_text::Action::Motion(cosmic_text::Motion::BufferEnd),
                                );
                                self.scroll_changed.set(true);
                            });

                            res = EventResponse::Sink;
                        }
                        KeyCode::KeyX if *down && main_modifier(modifiers) => {
                            let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                            if let Some(text) = editor.copy_selection() {
                                clipboard.copy(&text);
                            }
                            editor.delete_selection();
                            self.text_changed_by_cosmic.set(true);

                            res = EventResponse::Sink;
                        }
                        KeyCode::KeyC if *down && main_modifier(modifiers) => {
                            let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                            if let Some(text) = editor.copy_selection() {
                                clipboard.copy(&text);
                            }

                            res = EventResponse::Sink;
                        }
                        KeyCode::KeyV if *down && main_modifier(modifiers) => {
                            let clipboard = ctx.dom.get_global_or_init(ClipboardHolder::default);

                            if let Some(text) = clipboard.paste() {
                                editor.insert_string(&text, None);
                                self.text_changed_by_cosmic.set(true);
                            }

                            res = EventResponse::Sink;
                        }

                        _ => res = EventResponse::Sink,
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
                }

                res
            }
            WidgetEvent::TextPreedit(text, position) => {
                if let Some(editor) = self.cosmic_editor.get_mut() {
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
                }

                EventResponse::Sink
            }
            WidgetEvent::TextInput(c, modifiers) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                if !modifiers.ctrl() && !modifiers.meta() {
                    fonts.with_inner(|fonts| {
                        if let Some(editor) = self.cosmic_editor.get_mut() {
                            editor.action(&mut fonts.font_system, cosmic_text::Action::Insert(*c));
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

impl TextBoxWidget {
    fn get_offset(&self, scale_factor: f32) -> Vec2 {
        if self.props.min_width > 0.0 {
            let max_text_width = self.max_text_width.get();
            let min_width = self.props.min_width * scale_factor;

            let offset = match self.props.style.align {
                TextAlignment::Start => 0.0,
                TextAlignment::Center => (max_text_width - min_width) / 2.0,
                TextAlignment::End => max_text_width - min_width,
            };

            Vec2::new(-(offset).min(0.0), 0.0)
        } else {
            Vec2::ZERO
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
