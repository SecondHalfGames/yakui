#![allow(clippy::new_without_default, clippy::collapsible_else_if)]

#[macro_use]
mod mopmopa;

mod component;
mod event;
mod geometry;
mod input;
mod state;

pub mod context;
pub mod dom;
pub mod draw;
pub mod layout;

pub extern crate glam;
pub use glam::{Vec2, Vec4};
pub use thunderdome::Index;

pub use component::*;
pub use event::Event;
pub use geometry::*;
pub use input::*;
pub use state::State;
