use yakui_core::{FlexFit, Widget};

use crate::util::widget_children;

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

    fn flex(&self) -> (u32, FlexFit) {
        (self.props.flex, self.props.fit)
    }
}
