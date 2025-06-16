#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
    clippy::new_without_default,
)]
#![doc = include_str!("../README.md")]

mod ignore_debug;

pub mod util;

// Stubbed out implementation awaiting:
// https://github.com/1Password/arboard/pull/103
// https://github.com/1Password/arboard/pull/171
#[cfg_attr(
    not(any(target_os = "linux", target_os = "macos", target_os = "windows")),
    path = "clipboard_stub.rs"
)]
pub mod clipboard;

pub mod colors;
pub mod font;
pub mod shapes;
pub mod shorthand;
pub mod style;
pub mod text_renderer;
pub mod widgets;

pub use self::shorthand::*;

pub use cosmic_text;

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
