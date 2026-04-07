use std::borrow::Cow;
use std::cell::{Cell, RefCell};

use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::{PaintDom, PaintRect, Pipeline};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::auto_builders;
use crate::font::{FontSelection, Fonts};
use crate::style::{TextAlignment, TextStyle};
use crate::text_renderer::{GlyphRender, Kind, TextGlobalState};
use crate::util::widget;

/**
Renders text. You probably want to use [Text][super::Text] instead, which
supports features like padding.

Responds with [RenderTextResponse].
*/
#[derive(Debug, Clone, Default)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct RenderText {
    pub text: Cow<'static, str>,
    pub style: TextStyle,
    /// The text normally fills up the entire available space if it doesn't go from left to right,
    /// marking it as inline would force it to calculate the text width instead.
    pub inline: bool,
    pub min_width: f32,
}

auto_builders!(RenderText {
    inline: bool,
    min_width: f32,
});

pub struct RenderTextResponse {
    pub size: Option<Vec2>,
    pub max_text_size: Vec2,
}

impl RenderText {
    pub fn new<S: Into<Cow<'static, str>>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
            inline: false,
            min_width: 0.0,
        }
    }

    pub fn with_style<S: Into<Cow<'static, str>>>(text: S, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
            inline: false,
            min_width: 0.0,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<RenderTextResponse> {
        Self::show_with_scroll(self, None, None)
    }

    #[track_caller]
    pub fn show_with_scroll(
        self,
        scroll: Option<cosmic_text::Scroll>,
        cursor: Option<cosmic_text::Cursor>,
    ) -> Response<RenderTextResponse> {
        widget::<RenderTextWidget>((self, scroll, cursor))
    }
}

#[derive(Debug)]
pub struct RenderTextWidget {
    buffer: RefCell<Option<cosmic_text::Buffer>>,

    props: RenderText,
    text_changed: Cell<bool>,

    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,

    text_size: Cell<Option<Vec2>>,
    max_text_size: Cell<Vec2>,

    scroll: Option<cosmic_text::Scroll>,
    scroll_changed: Cell<bool>,
    cursor: Option<cosmic_text::Cursor>,
}

impl Widget for RenderTextWidget {
    type Props<'a> = (
        RenderText,
        Option<cosmic_text::Scroll>,
        Option<cosmic_text::Cursor>,
    );
    type Response = RenderTextResponse;

    fn new() -> Self {
        Self {
            buffer: RefCell::default(),

            props: RenderText::new(""),
            text_changed: Cell::new(true),

            max_size: Cell::default(),
            scale_factor: Cell::default(),

            text_size: Cell::default(),
            max_text_size: Cell::default(),

            scroll: None,
            scroll_changed: Cell::new(true),
            cursor: None,
        }
    }

    fn update(&mut self, (props, scroll, cursor): Self::Props<'_>) -> Self::Response {
        if props.text != self.props.text || props.style != self.props.style {
            self.text_changed.set(true);
        }

        if scroll != self.scroll {
            self.scroll_changed.set(true)
        }

        self.props = props;
        self.scroll = scroll;
        self.cursor = cursor;

        Self::Response {
            size: self.text_size.get(),
            max_text_size: self.max_text_size.get(),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let max_width = (constraints.min.x.max(constraints.max.x)).max(self.props.min_width);
        let max_width = if max_width.is_normal() {
            Some((max_width * ctx.layout.scale_factor()).ceil())
        } else {
            let max_width = self.props.min_width;
            if max_width.is_normal() {
                Some((max_width * ctx.layout.scale_factor()).ceil())
            } else {
                None
            }
        };

        let max_height = constraints.min.y.max(constraints.max.y);
        let max_height = if max_height.is_normal() {
            Some((max_height * ctx.layout.scale_factor()).ceil())
        } else {
            None
        };

        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        fonts.with_inner(|fonts| {
            let text_changed = self.text_changed.take();
            let scroll_changed = self.scroll_changed.take();
            let size_changed = self.max_size.get() != Some(max_size)
                || self.scale_factor.get() != Some(ctx.layout.scale_factor());

            self.max_size.set(Some(max_size));
            self.scale_factor.set(Some(ctx.layout.scale_factor()));

            let mut buffer_ref = self.buffer.borrow_mut();
            let buffer = buffer_ref.get_or_insert_with(|| {
                cosmic_text::Buffer::new(
                    &mut fonts.font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                )
            });

            if text_changed {
                buffer.set_text(
                    &mut fonts.font_system,
                    &self.props.text,
                    &fonts
                        .font_selection
                        .get_cosmic_attrs(&self.props.style.font),
                    cosmic_text::Shaping::Advanced,
                    self.props.style.align.into(),
                );
            }

            if text_changed || size_changed {
                let max_text_width = match max_width {
                    // if it's not inline, then we check if anything is RTL or if the text doesn't flow from left to right
                    Some(max_width)
                        if !self.props.inline
                            && (self.props.style.align != TextAlignment::Start
                                || buffer.layout_runs().any(|run| run.rtl)) =>
                    {
                        max_width.ceil()
                    }
                    // else, just measure the text's width since we have no reason take up the entire width
                    _ => measure_text_width(
                        &mut fonts.font_system,
                        &mut fonts.font_selection,
                        &self.props.text,
                        &self.props.style,
                        ctx.layout.scale_factor(),
                        max_width,
                    )
                    .max(self.props.min_width * ctx.layout.scale_factor())
                    .ceil(),
                };
                let max_text_height = measure_text_height(
                    &mut fonts.font_system,
                    &mut fonts.font_selection,
                    &self.props.text,
                    &self.props.style,
                    ctx.layout.scale_factor(),
                    max_width,
                )
                .min(max_height.unwrap_or(f32::INFINITY))
                .ceil();

                self.max_text_size
                    .set(Vec2::new(max_text_width, max_text_height));
                buffer.set_metrics(
                    &mut fonts.font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                );
                buffer.set_size(&mut fonts.font_system, Some(max_text_width), max_height);
            }

            if text_changed || size_changed || scroll_changed {
                if let Some(scroll) = self.scroll {
                    buffer.set_scroll(scroll);
                    buffer.shape_until_scroll(&mut fonts.font_system, text_changed);
                }

                let size = (self.max_text_size.get() / ctx.layout.scale_factor()).ceil();

                self.text_size.set(Some(size));
            }

            self.text_size.get().unwrap()
        })
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let widget_pos = ctx.layout.get(ctx.dom.current()).unwrap().rect.pos();

        let buffer_ref = self.buffer.borrow();
        let Some(buffer) = buffer_ref.as_ref() else {
            return;
        };

        let inv_scale_factor = 1.0 / ctx.layout.scale_factor();
        fonts.with_inner(|fonts| {
            let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);

            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let render =
                        text_global.get_glyph_render(ctx.paint, &mut fonts.font_system, glyph);
                    paint_text(
                        ctx.paint,
                        inv_scale_factor,
                        &run,
                        glyph,
                        render,
                        widget_pos,
                        self.props.style.color,
                    );
                }
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_text(
    paint: &mut PaintDom,
    inv_scale_factor: f32,
    run: &cosmic_text::LayoutRun,
    glyph: &cosmic_text::LayoutGlyph,
    render: GlyphRender,
    widget_pos: Vec2,
    color: Color,
) {
    let size = render.rect.size().as_vec2();

    let physical = glyph.physical((0.0, 0.0), 1.0);
    let pos = Vec2::new(physical.x as f32, physical.y as f32);

    let mut rect = PaintRect::new(Rect::from_pos_size(
        Vec2::new(
            pos.x + render.offset.x,
            pos.y - render.offset.y + run.line_y,
        ) * inv_scale_factor
            + widget_pos,
        Vec2::new(size.x, size.y) * inv_scale_factor,
    ));

    if render.kind == Kind::Mask {
        rect.color = color;
    } else {
        rect.color = Color::CLEAR;
    }
    rect.texture = Some((TextureId::Managed(render.texture), render.tex_rect));
    rect.pipeline = Pipeline::Text;

    rect.add(paint);
}

/// Measures the width of a particular string of text, with the given max width and FontSystem.
/// Wraps according to the max width.
///
/// Note: The returned width is in physical unit. The max width must also be in physical unit.
#[must_use]
pub fn measure_text_width(
    font_system: &mut cosmic_text::FontSystem,
    font_selection: &mut FontSelection,
    text: &str,
    style: &TextStyle,
    scale_factor: f32,
    max_width: Option<f32>,
) -> f32 {
    let mut buffer = cosmic_text::Buffer::new(font_system, style.to_metrics(scale_factor));
    buffer.set_size(font_system, max_width, None);
    buffer.set_text(
        font_system,
        text,
        &font_selection.get_cosmic_attrs(&style.font),
        cosmic_text::Shaping::Advanced,
        None,
    );
    buffer
        .layout_runs()
        .map(|run| {
            if run.rtl {
                max_width.unwrap_or(run.line_w)
            } else {
                run.line_w
            }
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or_default()
}

/// Measures the height of a particular string of text, with the given max width and FontSystem.
/// Wraps according to the max width.
///
/// Note: The returned height is in physical unit. The max width must also be in physical unit.
#[must_use]
pub fn measure_text_height(
    font_system: &mut cosmic_text::FontSystem,
    font_selection: &mut FontSelection,
    text: &str,
    style: &TextStyle,
    scale_factor: f32,
    max_width: Option<f32>,
) -> f32 {
    let mut buffer = cosmic_text::Buffer::new(font_system, style.to_metrics(scale_factor));
    buffer.set_size(font_system, max_width, None);
    buffer.set_text(
        font_system,
        text,
        &font_selection.get_cosmic_attrs(&style.font),
        cosmic_text::Shaping::Advanced,
        None,
    );

    buffer.layout_runs().map(|run| run.line_height).sum::<f32>()
}
