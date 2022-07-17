#![allow(clippy::new_without_default, clippy::collapsible_else_if)]
#![deny(missing_docs)]

//! TODO: Crate docs

#[macro_use]
mod mopmopa;

mod id;
mod response;
mod state;

pub mod context;
pub mod dom;
pub mod event;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod paint;
pub mod widget;

pub extern crate glam;

#[doc(no_inline)]
pub use glam::{UVec2, Vec2, Vec4};

pub use self::id::*;
pub use self::response::*;
pub use self::state::*;
