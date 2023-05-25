use yakui_core::widget::{PaintContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
Creates a new layer that will draw over the top of the current layer.
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Layer {}

impl Layer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<LayerWidget> {
        widget_children::<LayerWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct LayerWidget {
    props: Layer,
}

pub type LayerResponse = ();

impl Widget for LayerWidget {
    type Props = Layer;
    type Response = LayerResponse;

    fn new() -> Self {
        Self { props: Layer {} }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        ctx.paint.push_layer();

        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }

        ctx.paint.pop_layer();
    }
}
