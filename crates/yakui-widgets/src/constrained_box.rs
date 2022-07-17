use yakui_core::dom::Dom;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util::widget_children;

#[derive(Debug, Clone)]
pub struct ConstrainedBox {
    pub constraints: Constraints,
}

impl ConstrainedBox {
    pub fn new(constraints: Constraints) -> Self {
        Self { constraints }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ConstrainedBoxWidget> {
        widget_children::<ConstrainedBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ConstrainedBoxWidget {
    props: ConstrainedBox,
}

pub type ConstrainedBoxResponse = ();

impl Widget for ConstrainedBoxWidget {
    type Props = ConstrainedBox;
    type Response = ConstrainedBoxResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = Vec2::ZERO;
        let constraints = Constraints {
            min: Vec2::max(input.min, self.props.constraints.min),
            max: Vec2::min(input.max, self.props.constraints.max),
        };

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            size = size.max(child_size);
        }

        input.constrain(size)
    }

    fn respond(&mut self) -> Self::Response {}
}
