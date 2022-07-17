#![allow(clippy::new_without_default, clippy::collapsible_else_if)]
#![deny(missing_docs)]

//! TODO: Crate docs

#[macro_use]
mod mopmopa;

mod event;
mod geometry;
mod id;
mod response;
mod state;
mod widget;

pub mod context;
pub mod dom;
pub mod input;
pub mod layout;
pub mod paint;

pub extern crate glam;

#[doc(no_inline)]
pub use glam::{UVec2, Vec2, Vec4};

pub use self::event::*;
pub use self::geometry::*;
pub use self::id::*;
pub use self::response::*;
pub use self::state::*;
pub use self::widget::*;
