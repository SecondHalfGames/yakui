#![allow(clippy::new_without_default, clippy::collapsible_else_if)]

#[macro_use]
mod mopmopa;

mod event;
mod geometry;
mod input;
mod state;
mod widget;

pub mod context;
pub mod dom;
pub mod layout;
pub mod paint;

pub extern crate glam;
pub use glam::{UVec2, Vec2, Vec4};
pub use thunderdome::Index;

pub use event::*;
pub use geometry::*;
pub use input::*;
pub use state::*;
pub use widget::*;
