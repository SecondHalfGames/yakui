//! Defines how yakui responds to input and delegates it to widgets.

mod input_state;
mod mouse;

pub use self::input_state::*;
pub use self::mouse::*;

pub use keyboard_types::{Code as KeyCode, Modifiers};
