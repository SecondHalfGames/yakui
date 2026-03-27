use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::util::{widget, widget_children};
use crate::{auto_builders, shapes};

/**
A colored circle that can contain children.

Responds with [CircleResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Circle {
    pub color: Color,
    pub min_radius: f32,
}

auto_builders!(Circle {
    color: Color,
    min_radius: f32,
});

impl Circle {
    pub fn new() -> Self {
        Self {
            color: Color::WHITE,
            min_radius: 0.0,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<CircleResponse> {
        widget::<CircleWidget>(self)
    }

    #[track_caller]
    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<CircleResponse> {
        widget_children::<CircleWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct CircleWidget {
    props: Circle,
}

pub type CircleResponse = ();

impl Widget for CircleWidget {
    type Props<'a> = Circle;
    type Response = CircleResponse;

    fn new() -> Self {
        Self {
            props: Circle::new(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::splat(self.props.min_radius);

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let center = layout_node.rect.pos() + layout_node.rect.size() / 2.0;
        let radius = layout_node.rect.size().x.min(layout_node.rect.size().y) / 2.0;

        shapes::Circle::new(center, radius)
            .color(self.props.color)
            .add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
