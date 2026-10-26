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
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Spacer {
    pub flex: u32,
}

impl Spacer {
    pub fn new(flex: u32) -> Self {
        Self { flex: flex.max(1) }
    }

    #[track_caller]
    pub fn show(self) -> Response<SpacerResponse> {
        widget::<SpacerWidget>(self)
    }
}

#[derive(Debug)]
pub struct SpacerWidget;

pub type SpacerResponse = ();

impl Widget for SpacerWidget {
    type Props = Spacer;
    type Response = SpacerResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn flex(&self, props: &Self::Props) -> (u32, FlexFit) {
        (props.flex, FlexFit::Tight)
    }

    fn layout(
        &self,
        _props: &Self::Props,
        _ctx: LayoutContext<'_>,
        constraints: Constraints,
    ) -> Vec2 {
        // Flutter also implements it like this
        constraints.min
    }

    fn paint(&self, _props: &Self::Props, _: PaintContext<'_>) {
        // No children
    }
}
