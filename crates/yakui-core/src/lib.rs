#![allow(clippy::new_without_default, clippy::collapsible_else_if)]

#[macro_use]
mod mopmopa;

mod event;
mod geometry;
mod input;
mod response;
mod state;
mod widget;

pub mod context;
pub mod dom;
pub mod layout;
pub mod paint;

pub extern crate glam;
pub use glam::{UVec2, Vec2, Vec4};
pub use thunderdome::Index;

pub use self::event::*;
pub use self::geometry::*;
pub use self::input::*;
pub use self::response::Response;
pub use self::state::*;
pub use self::widget::*;
