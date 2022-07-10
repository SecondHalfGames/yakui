use crate::Widget;

/// Placeholder widget used internally to emplace a component without
/// initializing it yet.
#[derive(Debug)]
pub(crate) struct DummyWidget;

impl Widget for DummyWidget {
    type Props = ();
    type Response = ();

    #[inline]
    fn new(_props: Self::Props) -> Self {
        Self
    }

    #[inline]
    fn update(&mut self, _props: Self::Props) {}

    #[inline]
    fn respond(&mut self) {}
}
