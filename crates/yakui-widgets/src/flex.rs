use yakui_core::dom::Dom;
use yakui_core::layout::LayoutDom;
use yakui_core::{Constraints, Widget};

use crate::util::widget_children;
use crate::FlexFit;

#[derive(Debug)]
pub struct Flex {
    pub flex: u32,
    pub fit: FlexFit,
}

impl Flex {
    pub fn new(flex: u32) -> Self {
        Self {
            flex,
            fit: FlexFit::Loose,
        }
    }

    pub fn expanded() -> Self {
        Self {
            flex: 1,
            fit: FlexFit::Tight,
        }
    }

    pub fn show<F: FnOnce()>(self, children: F) {
        widget_children::<FlexWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct FlexWidget {
    props: Flex,
}

pub type FlexResponse = ();

impl Widget for FlexWidget {
    type Props = Flex;
    type Response = FlexResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn respond(&mut self) -> Self::Response {}

    fn flex(&self) -> u32 {
        self.props.flex
    }

    fn layout(
        &self,
        dom: &Dom,
        layout: &mut LayoutDom,
        constraints: Constraints,
    ) -> yakui_core::Vec2 {
        let node = dom.get_current();

        let child_constraints = match self.props.fit {
            FlexFit::Tight => Constraints {
                min: constraints.max,
                max: constraints.max,
            },
            FlexFit::Loose => constraints,
        };

        let mut size = yakui_core::Vec2::ZERO;
        for &child in &node.children {
            let child_size = layout.calculate(dom, child, child_constraints);
            size = size.max(child_size);
        }

        constraints.constrain(size)
    }
}
