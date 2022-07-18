#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
)]

mod icons;
mod text_renderer;
mod util;

pub mod colors;
pub mod shorthand;
pub mod widgets;

#[doc(inline)]
pub use self::shorthand::*;

#[doc(hidden)]
pub mod doc_yakui {
    pub use crate::*;
    pub use yakui_core::geometry::*;
    pub use yakui_core::*;
}

#[doc(hidden)]
pub struct DocTest {
    state: yakui_core::State,
}

impl DocTest {
    pub fn start() -> Self {
        let mut state = yakui_core::State::new();
        state.start();
        Self { state }
    }
}

impl Drop for DocTest {
    fn drop(&mut self) {
        self.state.finish();
    }
}
