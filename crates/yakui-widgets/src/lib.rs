#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
    clippy::new_without_default,
)]

mod ignore_debug;

pub mod colors;
pub mod font;
pub mod shapes;
pub mod shorthand;
pub mod style;
pub mod text_renderer;
pub mod util;
pub mod widgets;

pub use self::shorthand::*;

#[doc(hidden)]
pub struct DocTest {
    state: yakui_core::Yakui,
}

impl DocTest {
    pub fn start() -> Self {
        let mut state = yakui_core::Yakui::new();
        state.start();
        Self { state }
    }
}

impl Drop for DocTest {
    fn drop(&mut self) {
        self.state.finish();
    }
}
