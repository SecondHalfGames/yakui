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
