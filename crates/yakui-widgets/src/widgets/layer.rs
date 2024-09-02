use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
Creates a new layer that will take input priority and draw over items in the
containing layer.

In the future, this widget may be extended to support arbitrary transforms
applied to layers.
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Layer {}

impl Layer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<LayerResponse> {
        widget_children::<LayerWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct LayerWidget {
    props: Layer,
}

pub type LayerResponse = ();

impl Widget for LayerWidget {
    type Props<'a> = Layer;
    type Response = LayerResponse;

    fn new() -> Self {
        Self { props: Layer {} }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);

        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain_min(size)
    }
}
