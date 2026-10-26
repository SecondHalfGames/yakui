use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::auto_builders;

/// A horizontal divider line. Will take up the whole width of the parent.
///
/// Responds with [DividerResponse].
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Divider {
    /// The color of the divider.
    pub color: Color,
    /// The thickness of the divider.
    pub thickness: f32,
    /// The height of the divider.
    /// How much vertical space it takes up.
    pub height: f32,
    /// The indent of the divider from the left.
    pub indent: f32,
    /// The indent of the divider from the right.
    pub end_indent: f32,
}

auto_builders!(Divider {
    color: Color,
    thickness: f32,
    height: f32,
    indent: f32,
    end_indent: f32,
});

impl Divider {
    pub fn new(color: Color, height: f32, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            height,
            indent: 0.0,
            end_indent: 0.0,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<DividerResponse> {
        crate::util::widget::<DividerWidget>(self)
    }
}

#[derive(Debug)]
pub struct DividerWidget;

pub type DividerResponse = ();

impl Widget for DividerWidget {
    type Props = Divider;
    type Response = DividerResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn layout(&self, props: &Self::Props, _ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        Vec2::new(input.min.x, props.height.clamp(input.min.y, input.max.y))
    }

    fn paint(&self, props: &Self::Props, ctx: PaintContext<'_>) {
        // We get the parent's width during the paint phase because
        // using constraints.max.x is often useless as it is often infinite.

        let id = ctx.dom.current();
        let Some(parent) = ctx.dom.get(id).unwrap().parent else {
            return;
        };

        let parent_rect = ctx.layout.get(parent).unwrap().rect;
        let layout_rect = ctx.layout.get(id).unwrap().rect;

        let line_pos = Vec2::new(parent_rect.pos().x, layout_rect.pos().y)
            + Vec2::new(props.indent, (layout_rect.size().y - props.thickness) / 2.0);
        let line_size = Vec2::new(
            parent_rect.size().x - props.indent - props.end_indent,
            props.thickness,
        );

        let mut line_rect = PaintRect::new(Rect::from_pos_size(line_pos, line_size));
        line_rect.color = props.color;
        line_rect.add(ctx.paint);
    }
}
