use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

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
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
}

auto_builders!(RoundRect {
    color: Color,
    min_size: Vec2,
    top_left_radius: f32,
    top_right_radius: f32,
    bottom_left_radius: f32,
    bottom_right_radius: f32,
});

impl RoundRect {
    pub fn new(radius: f32) -> Self {
        Self {
            color: Color::WHITE,
            min_size: Vec2::ZERO,
            top_left_radius: radius,
            top_right_radius: radius,
            bottom_left_radius: radius,
            bottom_right_radius: radius,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.top_right_radius = radius;
        self.bottom_left_radius = radius;
        self.bottom_right_radius = radius;
        self
    }

    pub fn radii(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_left: f32,
        bottom_right: f32,
    ) -> Self {
        self.top_left_radius = top_left;
        self.top_right_radius = top_right;
        self.bottom_left_radius = bottom_left;
        self.bottom_right_radius = bottom_right;
        self
    }

    pub fn top_radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.top_right_radius = radius;
        self
    }

    pub fn bottom_radius(mut self, radius: f32) -> Self {
        self.bottom_left_radius = radius;
        self.bottom_right_radius = radius;
        self
    }

    pub fn left_radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.bottom_left_radius = radius;
        self
    }

    pub fn right_radius(mut self, radius: f32) -> Self {
        self.top_right_radius = radius;
        self.bottom_right_radius = radius;
        self
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

        let mut rect = shapes::RoundedRectangle::new(layout_node.rect, 0.0);
        rect.color = self.props.color;

        rect.top_left_radius = self.props.top_left_radius;
        rect.top_right_radius = self.props.top_right_radius;
        rect.bottom_left_radius = self.props.bottom_left_radius;
        rect.bottom_right_radius = self.props.bottom_right_radius;

        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
