use yakui_core::geometry::FlexFit;
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util::widget_children;

#[derive(Debug)]
pub struct Flexible {
    pub flex: u32,
    pub fit: FlexFit,
}

impl Flexible {
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

    pub fn show<F: FnOnce()>(self, children: F) -> Response<FlexWidget> {
        widget_children::<FlexWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct FlexWidget {
    props: Flexible,
}

pub type FlexResponse = ();

impl Widget for FlexWidget {
    type Props = Flexible;
    type Response = FlexResponse;

    fn new() -> Self {
        Self {
            props: Flexible::new(0),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        (self.props.flex, self.props.fit)
    }
}
