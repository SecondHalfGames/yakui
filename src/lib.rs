#![allow(clippy::new_without_default, clippy::collapsible_else_if)]

#[macro_use]
mod mopmopa;

mod component;
mod context;
mod dom;
mod event;
mod geometry;
mod state;
mod widgets;
mod zip_longest;

pub mod draw;

pub extern crate glam;
pub use glam::{Vec2, Vec4};

pub use event::Event;
pub use geometry::*;
pub use state::State;
pub use widgets::*;
