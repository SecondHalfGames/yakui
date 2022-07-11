use yakui_core::Widget;

use crate::util::widget_children;

#[derive(Debug)]
pub struct Flex {
    pub flex: f32,
}

impl Flex {
    pub fn new(flex: f32) -> Self {
        Self { flex }
    }

    pub fn expanded() -> Self {
        Self { flex: 1.0 }
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

    fn flex(&self) -> Option<f32> {
        Some(self.props.flex)
    }
}
