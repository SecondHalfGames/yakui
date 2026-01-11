use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::border::{Border, BorderRadius};
use crate::util::{widget, widget_children};
use crate::{auto_builders, shapes};

/**
A colored box with rounded corners that can contain children.

Responds with [RoundRectResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct RoundRect {
    pub color: Color,
    pub min_size: Vec2,
    pub border: Option<Border>,
    pub radius: BorderRadius,
}

auto_builders!(RoundRect {
    color: Color,
    min_size: Vec2,
    border: Option<Border>,
    radius: BorderRadius,
});

impl RoundRect {
    pub fn new<T: Into<BorderRadius>>(radius: T) -> Self {
        Self {
            color: Color::WHITE,
            min_size: Vec2::ZERO,
            radius: radius.into(),
            border: None,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<RoundRectResponse> {
        widget::<RoundRectWidget>(self)
    }

    #[track_caller]
    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<RoundRectResponse> {
        widget_children::<RoundRectWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct RoundRectWidget {
    props: RoundRect,
}

pub type RoundRectResponse = ();

impl Widget for RoundRectWidget {
    type Props<'a> = RoundRect;
    type Response = RoundRectResponse;

    fn new() -> Self {
        Self {
            props: RoundRect::new(0.0),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let mut rect = shapes::RoundedRectangle::new(layout_node.rect, self.props.radius);
        rect.color = self.props.color;
        rect.border = self.props.border;
        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
