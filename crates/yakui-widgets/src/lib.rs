#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
    clippy::new_without_default,
)]

mod icons;
mod text_renderer;

pub use text_renderer::{GlyphCache, LateBindingGlyphCache, TextGlobalState};
use yakui_core::paint::TextureReservation;

pub mod colors;
pub mod font;
pub mod shorthand;
pub mod style;
pub mod util;
pub mod widgets;

pub use self::shorthand::*;

#[doc(hidden)]
pub struct DocTest {
    state: yakui_core::State,
}

impl DocTest {
    pub fn start() -> Self {
        let mut state = yakui_core::State::new(|_| {
            // this is a dummy response
            (yakui_core::TextureId::new(0), TextureReservation::Completed)
        });
        state.start();
        Self { state }
    }
}

impl Drop for DocTest {
    fn drop(&mut self) {
        self.state.finish();
    }
}
