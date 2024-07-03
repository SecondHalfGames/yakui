use std::cell::{Cell, RefCell};
use std::fmt;
use std::sync::Arc;

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
pub struct RenderText {
    pub text: String,
    pub style: TextStyle,
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

    pub fn show(self) -> Response<Option<RenderTextBufferResponse>> {
        Self::show_with_scroll(self, None)
    }

    pub fn show_with_scroll(
        self,
        scroll: Option<cosmic_text::Scroll>,
    ) -> Response<Option<RenderTextBufferResponse>> {
        widget::<RenderTextWidget>((self, scroll))
    }
}

#[derive(Debug)]
pub struct RenderTextWidget {
    props: RenderText,
    last_text: RefCell<String>,
    buffer: RefCell<Option<Arc<cosmic_text::Buffer>>>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    scale_factor: Cell<Option<f32>>,
    last_scroll: Cell<Option<cosmic_text::Scroll>>,
    scroll: Option<cosmic_text::Scroll>,
}

impl Widget for RenderTextWidget {
    type Props<'a> = (RenderText, Option<cosmic_text::Scroll>);
    type Response = Option<RenderTextBufferResponse>;

    fn new() -> Self {
        Self {
            props: RenderText::new(""),
            last_text: RefCell::new(String::new()),
            buffer: RefCell::default(),
            max_size: Cell::default(),
            scale_factor: Cell::default(),
            last_scroll: Cell::default(),
            scroll: None,
        }
    }

    fn update(&mut self, (props, scroll): Self::Props<'_>) -> Self::Response {
        self.props = props;
        self.scroll = scroll;

        if let Some(buffer) = self.buffer.borrow().clone() {
            Some(widget::<RenderTextBufferWidget>((buffer, self.props.style.color)).into_inner())
        } else {
            None
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

        if self.buffer.borrow().is_none()
            || self.last_text.borrow().as_str() != self.props.text.as_str()
            || self.scale_factor.get() != Some(ctx.layout.scale_factor())
            || self.max_size.get() != Some(max_size)
            || self.last_scroll.get() != self.scroll
        {
            let fonts = ctx.dom.get_global_or_init(Fonts::default);

            fonts.with_system(|font_system| {
                let mut buffer = cosmic_text::Buffer::new(
                    font_system,
                    self.props.style.to_metrics(ctx.layout.scale_factor()),
                );

                buffer.set_size(font_system, max_width, max_height);

                buffer.set_text(
                    font_system,
                    &self.props.text,
                    self.props.style.attrs.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );

                if let Some(scroll) = self.scroll {
                    buffer.set_scroll(scroll);
                }

                buffer.shape_until_scroll(font_system, false);

                self.last_text.replace(self.props.text.clone());
                self.scale_factor.set(Some(ctx.layout.scale_factor()));
                self.max_size.replace(Some(max_size));
                self.last_scroll.replace(self.scroll);

                self.buffer.replace(Some(Arc::new(buffer)));
            });
        }

        self.default_layout(ctx, constraints)
    }
}

pub struct RenderTextBufferWidget {
    buffer: Option<Arc<cosmic_text::Buffer>>,
    color: Color,
    size: Cell<Vec2>,
}

pub struct RenderTextBufferResponse {
    pub size: Vec2,
}

impl Widget for RenderTextBufferWidget {
    type Props<'a> = (Arc<cosmic_text::Buffer>, Color);
    type Response = RenderTextBufferResponse;

    fn new() -> Self {
        Self {
            buffer: None,
            color: Color::WHITE,
            size: Cell::default(),
        }
    }

    fn update(&mut self, (buffer, color): Self::Props<'_>) -> Self::Response {
        self.buffer = Some(buffer);
        self.color = color;

        RenderTextBufferResponse {
            size: self.size.get(),
        }
    }

    fn layout(&self, _ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let Some(buffer) = self.buffer.clone() else {
            return Vec2::ZERO;
        };

        let size = {
            let size_x = buffer
                .layout_runs()
                .map(|layout| layout.line_w)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();

            let size_y = buffer.layout_runs().map(|layout| layout.line_height).sum();

            Vec2::new(size_x, size_y)
        };

        let size = constraints.constrain(size);

        self.size.set(size);

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let Some(buffer) = self.buffer.clone() else {
            return;
        };

        fonts.with_system(|font_system| {
            let text_global = ctx.dom.get_global_or_init(TextGlobalState::new);

            for layout in buffer.layout_runs() {
                for glyph in layout.glyphs {
                    if let Some(render) = text_global.get_or_insert(ctx.paint, font_system, glyph) {
                        paint_text(
                            &mut ctx,
                            self.color,
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
    let size = render.rect.size().as_vec2();
    let physical = glyph.physical((0.0, 0.0), 1.0);
    let pos = Vec2::new(physical.x as f32, physical.y as f32);

    let mut rect = PaintRect::new(Rect::from_pos_size(
        Vec2::new(pos.x + render.offset.x, pos.y - render.offset.y + line_y) + layout_pos,
        Vec2::new(size.x, size.y),
    ));

    if render.kind == Kind::Mask {
        rect.color = color;
    } else {
        rect.color = Color::rgba(0, 0, 0, 255);
    }
    rect.texture = Some((TextureId::Managed(render.texture), render.tex_rect));
    rect.pipeline = Pipeline::Text;

    rect.add(ctx.paint);
}

impl fmt::Debug for RenderTextBufferWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderTextBufferWidget").finish()
    }
}
