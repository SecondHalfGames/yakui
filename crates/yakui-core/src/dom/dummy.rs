use crate::widget::Widget;

/// Placeholder widget used internally to emplace a widget without
/// initializing it yet.
#[derive(Debug)]
pub(crate) struct DummyWidget;

impl Widget for DummyWidget {
    type Props<'a> = ();
    type Response = ();

    #[inline]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}
}
