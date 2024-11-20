use std::cell::{Cell, RefCell};

use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::{PaintRect, Pipeline};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::font::Fonts;
use crate::style::TextStyle;
use crate::text_renderer::{GlyphRender, Kind, TextGlobalState};
use crate::util::widget;

/**
Renders text. You probably want to use [Text][super::Text] instead, which
supports features like padding.

Responds with [RenderTextBoxResponse].
*/
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
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

    pub fn show(self) -> Response<RenderTextResponse> {
        Self::show_with_scroll(self, None)
    }

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
    size: Cell<Option<Vec2>>,
    last_text: RefCell<String>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,
    last_scroll: Cell<Option<cosmic_text::Scroll>>,
    scroll: Option<cosmic_text::Scroll>,
}

impl Widget for RenderTextWidget {
    type Props<'a> = (RenderText, Option<cosmic_text::Scroll>);
    type Response = RenderTextResponse;

    fn new() -> Self {
        Self {
            props: RenderText::new(""),
            buffer: RefCell::default(),
            size: Cell::default(),
            last_text: RefCell::new(String::new()),
            max_size: Cell::default(),
            scale_factor: Cell::default(),
            last_scroll: Cell::default(),
            scroll: None,
        }
    }

    fn update(&mut self, (props, scroll): Self::Props<'_>) -> Self::Response {
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
            let mut buffer = self.buffer.take().unwrap_or_else(|| {
                cosmic_text::Buffer::new(
                    font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                )
            });

            if self.scale_factor.get() != Some(ctx.layout.scale_factor())
                || self.max_size.get() != Some(max_size)
            {
                buffer.set_metrics_and_size(
                    font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                    max_width,
                    max_height,
                );

                self.max_size.replace(Some(max_size));
                self.scale_factor.set(Some(ctx.layout.scale_factor()));
            }

            if self.last_scroll.get() != self.scroll {
                if let Some(scroll) = self.scroll {
                    buffer.set_scroll(scroll);
                }

                self.last_scroll.replace(self.scroll);
            }

            if self.last_text.borrow().as_str() != self.props.text.as_str() {
                buffer.set_text(
                    font_system,
                    &self.props.text,
                    self.props.style.attrs.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );

                self.last_text.replace(self.props.text.clone());
            }

            // Perf note: https://github.com/pop-os/cosmic-text/issues/166
            for buffer_line in buffer.lines.iter_mut() {
                buffer_line.set_align(Some(self.props.style.align.into()));
            }

            buffer.shape_until_scroll(font_system, true);

            let size = {
                let size_x = buffer
                    .layout_runs()
                    .map(|layout| layout.line_w)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or_default()
                    .ceil();

                let size_y = buffer
                    .layout_runs()
                    .map(|layout| layout.line_height)
                    .sum::<f32>()
                    .ceil();

                (Vec2::new(size_x, size_y) / ctx.layout.scale_factor()).round()
            };

            let size = constraints.constrain(size);
            self.size.set(Some(size));

            self.buffer.replace(Some(buffer));

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
            let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);

            for layout in buffer.layout_runs() {
                for glyph in layout.glyphs {
                    if let Some(render) = text_global.get_or_insert(ctx.paint, font_system, glyph) {
                        paint_text(
                            &mut ctx,
                            self.props.style.color,
                            glyph,
                            render,
                            layout_node.rect.pos(),
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
