use yakui_core::{
    geometry::{Constraints, FlexFit, Vec2},
    widget::{LayoutContext, Widget},
    Response,
};

/// Occupies the largest possible space, draws nothing, and consumes no inputs
///
/// Capture `Viewport.show().id`, then look it up in the final `LayoutDom` to find its
/// dimensions. Useful identifying a viewport for non-GUI rendering.
#[derive(Debug, Default)]
pub struct Viewport;

impl Viewport {
    pub fn show(&self) -> Response<Self> {
        crate::util::widget::<Self>(())
    }
}

impl Widget for Viewport {
    type Props = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, (): ()) {}

    fn layout(&self, _: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        constraints
            .max
            .as_ref()
            .map(|x| match x.is_infinite() {
                true => 0.0,
                false => x,
            })
            .into()
    }

    fn flex(&self) -> (u32, FlexFit) {
        (1, FlexFit::Tight)
    }
}
