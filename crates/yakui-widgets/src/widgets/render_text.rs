use std::cell::{Cell, RefCell};

use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::{PaintRect, Pipeline};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::font::Fonts;
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
    pub text: String,
    pub style: TextStyle,
}

pub struct RenderTextResponse {
    pub size: Option<Vec2>,
}

impl RenderText {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label(),
        }
    }

    pub fn with_style<S: Into<String>>(text: S, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<RenderTextResponse> {
        Self::show_with_scroll(self, None)
    }

    #[track_caller]
    pub fn show_with_scroll(
        self,
        scroll: Option<cosmic_text::Scroll>,
    ) -> Response<RenderTextResponse> {
        widget::<RenderTextWidget>((self, scroll))
    }
}

#[derive(Debug)]
pub struct RenderTextWidget {
    props: RenderText,
    buffer: RefCell<Option<cosmic_text::Buffer>>,
    line_offsets: RefCell<Vec<f32>>,
    size: Cell<Option<Vec2>>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,
    last_scroll: Cell<Option<cosmic_text::Scroll>>,
    scroll: Option<cosmic_text::Scroll>,
    relayout: Cell<bool>,
}

impl Widget for RenderTextWidget {
    type Props<'a> = (RenderText, Option<cosmic_text::Scroll>);
    type Response = RenderTextResponse;

    fn new() -> Self {
        Self {
            props: RenderText::new(""),
            buffer: RefCell::default(),
            line_offsets: RefCell::default(),
            size: Cell::default(),
            max_size: Cell::default(),
            scale_factor: Cell::default(),
            last_scroll: Cell::default(),
            scroll: None,
            relayout: Cell::new(false),
        }
    }

    fn update(&mut self, (props, scroll): Self::Props<'_>) -> Self::Response {
        if props.text != self.props.text || props.style.attrs != self.props.style.attrs {
            self.relayout.set(true);
        }

        self.props = props;
        self.scroll = scroll;

        Self::Response {
            size: self.size.get(),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let max_width = constraints
            .max
            .x
            .is_finite()
            .then_some(constraints.max.x * ctx.layout.scale_factor());
        let max_height = constraints
            .max
            .y
            .is_finite()
            .then_some(constraints.max.y * ctx.layout.scale_factor());
        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        fonts.with_system(|font_system| {
            let relayout = self.relayout.take();

            let mut buffer_ref = self.buffer.borrow_mut();
            let buffer = buffer_ref.get_or_insert_with(|| {
                cosmic_text::Buffer::new(
                    font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                )
            });

            if self.scale_factor.get() != Some(ctx.layout.scale_factor())
                || self.max_size.get() != Some(max_size)
                || relayout
            {
                buffer.set_metrics_and_size(
                    font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                    max_width,
                    max_height,
                );

                self.max_size.set(Some(max_size));
                self.scale_factor.set(Some(ctx.layout.scale_factor()));
            }

            if self.last_scroll.get() != self.scroll {
                if let Some(scroll) = self.scroll {
                    buffer.set_scroll(scroll);
                }

                self.last_scroll.set(self.scroll);
            }

            if relayout {
                buffer.set_text(
                    font_system,
                    &self.props.text,
                    &self.props.style.attrs.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );
            }

            buffer.shape_until_scroll(font_system, true);

            let mut line_offsets = self.line_offsets.borrow_mut();
            line_offsets.clear();

            let widest_line = buffer
                .layout_runs()
                .map(|layout| layout.line_w)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default()
                .ceil()
                .max(constraints.min.x * ctx.layout.scale_factor());

            for run in buffer.layout_runs() {
                let offset = match self.props.style.align {
                    TextAlignment::Start => 0.0,
                    TextAlignment::Center => (widest_line - run.line_w) / 2.0,
                    TextAlignment::End => widest_line - run.line_w,
                };

                line_offsets.push(offset / ctx.layout.scale_factor());
            }

            let size = {
                let size_y = buffer
                    .layout_runs()
                    .map(|layout| layout.line_height)
                    .sum::<f32>()
                    .ceil();

                (Vec2::new(widest_line, size_y) / ctx.layout.scale_factor()).round()
            };

            let size = constraints.constrain(size);
            self.size.set(Some(size));

            size
        })
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let buffer_ref = self.buffer.borrow();
        let Some(buffer) = buffer_ref.as_ref() else {
            return;
        };

        fonts.with_system(|font_system| {
            let line_offsets = self.line_offsets.borrow();
            let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);

            for (layout, x_offset) in buffer.layout_runs().zip(line_offsets.iter().copied()) {
                for glyph in layout.glyphs {
                    if let Some(render) = text_global.get_or_insert(ctx.paint, font_system, glyph) {
                        paint_text(
                            &mut ctx,
                            self.props.style.color,
                            glyph,
                            render,
                            layout_node.rect.pos() + Vec2::new(x_offset, 0.0),
                            layout.line_y,
                        )
                    }
                }
            }
        });
    }
}

fn paint_text(
    ctx: &mut PaintContext<'_>,
    color: Color,
    glyph: &cosmic_text::LayoutGlyph,
    render: GlyphRender,
    layout_pos: Vec2,
    line_y: f32,
) {
    let inv_scale_factor = 1.0 / ctx.layout.scale_factor();

    let size = render.rect.size().as_vec2();

    let physical = glyph.physical((0.0, 0.0), 1.0);
    let pos = Vec2::new(physical.x as f32, physical.y as f32);

    let mut rect = PaintRect::new(Rect::from_pos_size(
        Vec2::new(pos.x + render.offset.x, pos.y - render.offset.y + line_y) * inv_scale_factor
            + layout_pos,
        Vec2::new(size.x, size.y) * inv_scale_factor,
    ));

    if render.kind == Kind::Mask {
        rect.color = color;
    } else {
        rect.color = Color::CLEAR;
    }
    rect.texture = Some((TextureId::Managed(render.texture), render.tex_rect));
    rect.pipeline = Pipeline::Text;

    rect.add(ctx.paint);
}
