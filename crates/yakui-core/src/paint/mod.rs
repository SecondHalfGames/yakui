//! Defines primitives for painting widgets, including the Paint DOM.

mod layers;
mod paint_dom;
mod primitives;
mod rect;
mod texture;

pub use self::layers::*;
pub use self::paint_dom::*;
pub use self::primitives::*;
pub use self::rect::PaintRect;
pub use self::texture::*;
