#![allow(
    clippy::new_without_default,
    clippy::collapsible_else_if,
    clippy::collapsible_if
)]
#![deny(missing_docs)]

//! TODO: Crate docs

#[macro_use]
mod mopmopa;

mod id;
mod response;
mod state;
mod types;

pub mod context;
pub mod dom;
pub mod event;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod paint;
pub mod widget;

pub use self::id::*;
pub use self::response::*;
pub use self::state::*;
pub use self::types::*;
