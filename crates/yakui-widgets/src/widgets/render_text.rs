use std::cell::{Cell, RefCell};

use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::auto_builders;
use crate::font::Fonts;
use crate::style::{TextAlignment, TextStyle};
use crate::text_renderer::{GlyphRender, Kind, TextGlobalState};
use crate::util::widget;

/**
Renders text. You probably want to use [Text][super::Text] instead, which
supports features like padding.

Responds with [RenderTextResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct RenderText {
    /// The text normally fills up the entire available space if it doesn't go from left to right,
    /// marking it as inline would force it to calculate the text width instead.
    pub inline: bool,
    pub min_width: f32,

    pub cursor: Option<cosmic_text::Cursor>,
    pub selection: Option<(cosmic_text::Cursor, cosmic_text::Cursor)>,
    pub preedit_cursor: Option<cosmic_text::Cursor>,
    pub preedit_cursor_end: Option<cosmic_text::Cursor>,
    pub selected_bg_color: Color,
    pub cursor_color: Color,
}

impl Default for RenderText {
    fn default() -> Self {
        Self {
            inline: false,
            min_width: Default::default(),

            cursor: None,
            selection: None,
            preedit_cursor: None,
            preedit_cursor_end: None,
            selected_bg_color: Color::CLEAR,
            cursor_color: Color::CLEAR,
        }
    }
}

auto_builders!(RenderText {
    inline: bool,
    min_width: f32,
});

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderTextResponse {
    /// The measured max size of the text, without accounting for constraints, in *physical* pixel unit.
    pub measured_size: Vec2,
    /// The size of the text, in *physical* pixel unit.
    pub physical_size: Vec2,
    /// The size of the text, in *logical* pixel unit.
    pub size: Vec2,
    /// The size constraints for the text, in *physical* pixel unit.
    pub max_size: (Option<f32>, Option<f32>),
}

impl RenderText {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[track_caller]
    pub fn show(self, text: &str, style: &TextStyle) -> Response<RenderTextResponse> {
        Self::show_with_scroll(self, text, style, None)
    }

    #[track_caller]
    pub fn show_with_scroll(
        self,
        text: &str,
        style: &TextStyle,
        scroll: Option<cosmic_text::Scroll>,
    ) -> Response<RenderTextResponse> {
        widget::<RenderTextWidget>((self, text, style, scroll))
    }
}

#[derive(Debug)]
pub struct RenderTextWidget {
    buffer: RefCell<Option<cosmic_text::Buffer>>,

    props: RenderText,

    text: String,
    style: TextStyle,
    scroll: Option<cosmic_text::Scroll>,
    text_changed: Cell<bool>,
    font_changed: Cell<bool>,
    align_changed: Cell<bool>,
    style_changed: Cell<bool>,
    scroll_changed: Cell<bool>,

    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,

    measured_size: Cell<Vec2>,
    physical_text_size: Cell<Vec2>,
    text_size: Cell<Vec2>,

    text_cursor: Cell<Option<(Rect, i32)>>,
}

impl Widget for RenderTextWidget {
    type Props<'a> = (
        RenderText,
        &'a str,
        &'a TextStyle,
        Option<cosmic_text::Scroll>,
    );
    type Response = RenderTextResponse;

    fn new() -> Self {
        Self {
            buffer: RefCell::default(),

            props: RenderText::new(),
            text: String::default(),
            style: TextStyle::default(),
            scroll: None,
            text_changed: Cell::new(true),
            font_changed: Cell::new(true),
            align_changed: Cell::new(true),
            style_changed: Cell::new(true),
            scroll_changed: Cell::new(false),

            max_size: Cell::default(),
            scale_factor: Cell::default(),

            measured_size: Cell::default(),
            physical_text_size: Cell::default(),
            text_size: Cell::default(),

            text_cursor: Cell::default(),
        }
    }

    fn update(&mut self, (props, text, style, scroll): Self::Props<'_>) -> Self::Response {
        if text != self.text {
            self.text = text.to_string();
            self.text_changed.set(true);
        }

        if style.font != self.style.font {
            self.style.font = style.font.clone();
            self.font_changed.set(true);
        }

        if style.align != self.style.align {
            self.style.align = style.align;
            self.align_changed.set(true);
        }

        if style != &self.style {
            self.style = style.clone();
            self.style_changed.set(true);
        }

        if scroll != self.scroll {
            self.scroll_changed.set(true)
        }

        self.props = props;
        self.scroll = scroll;

        Self::Response {
            measured_size: self.measured_size.get(),
            physical_size: self.physical_text_size.get(),
            size: self.text_size.get(),
            max_size: self.max_size.get().unwrap_or_default(),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        // if we're scrolling via cosmic-text, enable clipping
        if self.scroll.is_some() {
            ctx.layout.enable_clipping(ctx.dom);
        }

        if let Some(cursor) = self.text_cursor.get() {
            ctx.input.set_text_cursor(cursor);
        }

        let max_width = (constraints.min.x.max(constraints.max.x)).max(self.props.min_width);
        let max_width = if max_width.is_normal() {
            Some(max_width * ctx.layout.scale_factor())
        } else {
            let max_width = self.props.min_width;
            if max_width.is_normal() {
                Some(max_width * ctx.layout.scale_factor())
            } else {
                None
            }
        };

        let max_height = constraints.min.y.max(constraints.max.y);
        let max_height = if max_height.is_normal() {
            Some(max_height * ctx.layout.scale_factor())
        } else {
            None
        };

        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        fonts.with_inner(|fonts| {
            let text_changed = self.text_changed.take();
            let font_changed = self.font_changed.take();
            let align_changed = self.align_changed.take();
            let style_changed = self.style_changed.take();
            let scroll_changed = self.scroll_changed.take();

            let max_size_changed = self.max_size.get() != Some(max_size);
            let scale_changed = self.scale_factor.get() != Some(ctx.layout.scale_factor());
            self.max_size.set(Some(max_size));
            self.scale_factor.set(Some(ctx.layout.scale_factor()));

            let mut buffer_ref = self.buffer.borrow_mut();
            let buffer = buffer_ref.get_or_insert_with(|| {
                cosmic_text::Buffer::new(
                    &mut fonts.font_system,
                    self.style.to_metrics(ctx.layout.scale_factor()),
                )
            });

            // set_text is potentially expensive.
            // only update the styles if it's not necessary to set the text.
            if text_changed || font_changed {
                buffer.set_text(
                    &self.text,
                    &fonts.font_selection.get_cosmic_attrs(&self.style.font),
                    cosmic_text::Shaping::Advanced,
                    self.style.align.into(),
                );
            } else if align_changed {
                for line in &mut buffer.lines {
                    line.set_align(self.style.align.into());
                }
            }

            if style_changed || scale_changed {
                buffer.set_metrics(self.style.to_metrics(ctx.layout.scale_factor()));
            }

            if text_changed || font_changed || style_changed || max_size_changed || scale_changed {
                let (measured_width, measured_height) =
                    measure_text_size(buffer, &mut fonts.font_system, max_width);

                let physical_text_width = match max_width {
                    // if it's not inline, then we check if anything is RTL or if the text doesn't flow from left to right
                    Some(max_width)
                        if !self.props.inline
                            && (self.style.align != TextAlignment::Start
                                || buffer.layout_runs().any(|run| run.rtl)) =>
                    {
                        max_width
                    }
                    // else, just measure the text's width since we have no reason take up the entire width
                    _ => measured_width.max(self.props.min_width * ctx.layout.scale_factor()),
                };

                self.measured_size
                    .set(Vec2::new(measured_width, measured_height));
                self.physical_text_size.set(Vec2::new(
                    physical_text_width,
                    measured_height.min(max_height.unwrap_or(f32::INFINITY)),
                ));
                self.text_size
                    .set((self.physical_text_size.get() / ctx.layout.scale_factor()).ceil());

                buffer.set_size(Some(self.physical_text_size.get().x), max_height);
            }

            if text_changed
                || font_changed
                || align_changed
                || style_changed
                || scroll_changed
                || max_size_changed
                || scale_changed
            {
                if let Some(scroll) = self.scroll {
                    buffer.set_scroll(scroll);
                }

                buffer.shape_until_scroll(&mut fonts.font_system, text_changed || font_changed);
            }

            self.text_size.get()
        })
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let buffer_ref = self.buffer.borrow();
        let Some(buffer) = buffer_ref.as_ref() else {
            return;
        };

        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let layout_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let physical_layout_pos = layout_rect.pos() * ctx.layout.scale_factor();
        let inv_scale_factor = 1.0 / ctx.layout.scale_factor();
        fonts.with_inner(|fonts| {
            let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);

            if let Some((start, end)) = self.props.selection {
                for run in buffer.layout_runs() {
                    if !(start.line..=end.line).contains(&run.line_i) {
                        continue;
                    }

                    let y = run.line_top;
                    let h = run.line_height;

                    for (x, w) in run.highlight(start, end) {
                        let pos = Vec2::new(x, y) * inv_scale_factor;
                        let size = Vec2::new(w, h) * inv_scale_factor;

                        let mut selection_rect =
                            PaintRect::new(Rect::from_pos_size(layout_rect.pos() + pos, size));
                        selection_rect.color = self.props.selected_bg_color;
                        selection_rect.add(ctx.paint);
                    }
                }
            }

            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let (render, physical_glyph) = text_global.get_glyph_render(
                        ctx.paint,
                        &mut fonts.font_system,
                        glyph,
                        (physical_layout_pos.x, physical_layout_pos.y + run.line_y),
                    );
                    paint_text(ctx.paint, render, &physical_glyph, self.style.color);
                }
            }

            if let Some(cursor) = self.props.cursor {
                const CURSOR_WIDTH: f32 = 1.5;
                let line_width = self.physical_text_size.get().x;

                let cursor_rect = buffer.layout_runs().find_map(|run| {
                    let (mut x, y) = (run.cursor_position(&cursor)?, run.line_top);
                    let (w, h) = (run.line_w, run.line_height);

                    // Madeline Sparkles: when there's no glyphs that means the line is empty. cosmic_text doesn't
                    // put the cursor correctly when there's alignment.
                    // this is probably a bug on cosmic's part...
                    // the code below is copied from how cosmic text calculates text alignment
                    if run.glyphs.is_empty() {
                        if let Some(align) = buffer.lines[run.line_i].align() {
                            use cosmic_text::Align;

                            let alignment_correction = match (align, run.rtl) {
                                (Align::Left, true) => (line_width - run.line_w).max(0.),
                                (Align::Left, false) => 0.,
                                (Align::Right, true) => 0.,
                                (Align::Right, false) => (line_width - run.line_w).max(0.),
                                (Align::Center, _) => (line_width - run.line_w).max(0.) / 2.0,
                                (Align::End, _) => (line_width - run.line_w).max(0.),
                                (Align::Justified, _) => 0.,
                            };

                            x += alignment_correction;
                        }
                    }

                    Some(((x, y), (w, h)))
                });

                self.text_cursor.set(cursor_rect.map(|((x, y), (w, h))| {
                    (
                        Rect::from_pos_size(
                            physical_layout_pos + Vec2::new(0.0, y),
                            Vec2::new(w, h),
                        ),
                        x as i32,
                    )
                }));

                if let Some(((x, y), (_, h))) = cursor_rect {
                    let pos = Vec2::new(x * inv_scale_factor, y * inv_scale_factor);
                    let size = Vec2::new(CURSOR_WIDTH, h * inv_scale_factor);
                    let mut rect = Rect::from_pos_size(layout_rect.pos() + pos, size);
                    let diff_pos = (rect.pos().x - layout_rect.pos().x).min(0.0);
                    let diff_max = (layout_rect.max().x - rect.max().x).min(0.0);
                    rect.set_pos((rect.pos() + Vec2::new(-diff_pos + diff_max, 0.0)).floor());

                    let mut cursor_highlight = PaintRect::new(rect);
                    cursor_highlight.color = self.props.cursor_color;
                    cursor_highlight.add(ctx.paint);
                }
            }

            if let Some((preedit_cursor, preedit_cursor_end)) =
                self.props.preedit_cursor.zip(self.props.preedit_cursor_end)
            {
                for run in buffer.layout_runs() {
                    if !(preedit_cursor.line..=preedit_cursor_end.line).contains(&run.line_i) {
                        continue;
                    }

                    let y = run.line_top;
                    let h = run.line_height;

                    for (x, w) in run.highlight(preedit_cursor, preedit_cursor_end) {
                        let pos = Vec2::new(x, y + h) * inv_scale_factor - Vec2::new(0.0, 3.0);
                        let size = Vec2::new(w * inv_scale_factor, 3.0);

                        let mut preedit_highlight =
                            PaintRect::new(Rect::from_pos_size(layout_rect.pos() + pos, size));
                        preedit_highlight.color = self.style.color;
                        preedit_highlight.add(ctx.paint);
                    }
                }
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_text(
    paint: &mut PaintDom,
    render: GlyphRender,
    physical: &cosmic_text::PhysicalGlyph,
    color: Color,
) {
    // we supply the text pipeline with physical pixels.
    // scale factor is already accounted for by cosmic-text.
    // in the text pipeline, we'll skip scaling the vertices by the scale factor.
    let mut rect = PaintRect::new(Rect::from_pos_size(
        Vec2::new(
            (physical.x + render.atlas.offset.x) as f32,
            (physical.y - render.atlas.offset.y) as f32,
        ),
        render.atlas.size,
    ));

    if render.kind == Kind::Mask {
        rect.color = color;
    } else {
        rect.color = Color::CLEAR;
    }
    rect.texture = Some((TextureId::Managed(render.texture), render.atlas.tex_rect));
    rect.pipeline = Pipeline::Text;

    rect.add(paint);
}

/// Measures the width and height of a particular string of text, with the given max width and FontSystem.
/// Wraps according to the max width.
///
/// Note: The returned sizes are in physical unit. The max width must also be in physical unit.
#[must_use]
pub fn measure_text_size(
    buffer: &mut cosmic_text::Buffer,
    font_system: &mut cosmic_text::FontSystem,
    max_width: Option<f32>,
) -> (f32, f32) {
    let size = buffer.size();
    let scroll = buffer.scroll();

    let result = {
        buffer.set_size(max_width, None);
        buffer.set_scroll(cosmic_text::Scroll {
            line: usize::MAX,
            vertical: 0.0,
            horizontal: 0.0,
        });
        buffer.shape_until_scroll(font_system, false);

        let width = buffer
            .layout_runs()
            .map(|run| {
                if run.rtl {
                    max_width.unwrap_or(run.line_w)
                } else {
                    run.line_w
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_default();

        let height = buffer.layout_runs().map(|run| run.line_height).sum::<f32>();

        (width, height)
    };

    buffer.set_size(size.0, size.1);
    buffer.set_scroll(scroll);

    result
}
