//! Defines common geometry types used by yakui.

mod color;
mod constraints;
mod dim;
mod rect;
mod urect;

#[doc(no_inline)]
pub use glam::{UVec2, Vec2, Vec4};

pub use self::color::*;
pub use self::constraints::*;
pub use self::dim::*;
pub use self::rect::*;
pub use self::urect::*;

/// Defines how a flexible container should size its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexFit {
    /// The container forces its children to stretch to its size.
    Tight,

    /// The container lets the child have any size that fits within the
    /// container.
    Loose,
}
