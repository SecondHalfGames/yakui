#![allow(clippy::new_without_default)]

mod component;
mod context;
mod dom;
mod event;
mod rect;
mod snapshot;
mod state;
mod widgets;
mod zip_longest;

pub mod layout;

pub extern crate glam;
pub use glam::Vec2;

pub use event::Event;
pub use rect::Rect;
pub use state::State;
pub use widgets::*;
