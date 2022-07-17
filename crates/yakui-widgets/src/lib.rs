#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
)]

mod align;
mod button;
mod colored_box;
mod constrained_box;
mod flex;
mod image;
mod list;
mod padding;
mod text;
mod text_renderer;
mod util;
mod window;

pub mod colors;
pub mod shorthand;
pub mod types;

#[doc(inline)]
pub use self::shorthand::*;

pub mod widgets {
    pub use crate::align::*;
    pub use crate::button::*;
    pub use crate::colored_box::*;
    pub use crate::constrained_box::*;
    pub use crate::flex::*;
    pub use crate::image::*;
    pub use crate::list::*;
    pub use crate::padding::*;
    pub use crate::text::*;
    pub use crate::window::*;
}
