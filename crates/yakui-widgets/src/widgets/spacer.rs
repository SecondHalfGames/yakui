use yakui_core::geometry::{Constraints, FlexFit, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::util::widget;

/// A widget that takes up space as a flex element.
///
/// Expands to take up space with a provided flex factor. (usually 1)
///
/// Responds with [SpacerResponse].
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Spacer {
    pub flex: u32,
}

impl Spacer {
    pub fn new(flex: u32) -> Self {
        Self { flex: flex.max(1) }
    }

    pub fn show(self) -> Response<SpacerResponse> {
        widget::<SpacerWidget>(self)
    }
}

#[derive(Debug)]
pub struct SpacerWidget {
    props: Spacer,
}

pub type SpacerResponse = ();

impl Widget for SpacerWidget {
    type Props<'a> = Spacer;
    type Response = SpacerResponse;

    fn new() -> Self {
        Self {
            props: Spacer::new(1),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        (self.props.flex, FlexFit::Tight)
    }

    fn layout(&self, _ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        // Flutter also implements it like this
        constraints.min
    }

    fn paint(&self, _: PaintContext<'_>) {
        // No children
    }
}
