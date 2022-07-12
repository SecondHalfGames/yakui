#![allow(
    clippy::let_unit_value, // When implementing widgets, we want to spell
                            // out our Response type even if it's unit.
)]

mod align;
mod button;
mod colored_box;
mod constrained_box;
mod flex;
mod image;
mod list;
mod padding;
mod shorthand;
mod text;
mod text_renderer;
mod util;
mod window;

pub use shorthand::*;

pub use self::align::*;
pub use self::button::*;
pub use self::colored_box::*;
pub use self::constrained_box::*;
pub use self::flex::*;
pub use self::image::*;
pub use self::list::*;
pub use self::padding::*;
pub use self::text::*;
pub use self::window::*;

use yakui_core::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainAxisSize {
    Max,
    // Min,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainAxisAlignment {
    Start,
    // Center,
    // End,
    // SpaceAround,
    // SpaceBetween,
    // SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossAxisAlignment {
    Start,
    // Center,
    // End,
    // Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Right,
}

impl Direction {
    fn vec2(&self, main: f32, cross: f32) -> Vec2 {
        match self {
            Self::Down => Vec2::new(cross, main),
            Self::Right => Vec2::new(main, cross),
        }
    }

    fn get_main_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.y,
            Self::Right => vec.x,
        }
    }

    fn get_cross_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.x,
            Self::Right => vec.y,
        }
    }

    fn only_main_axis(&self, vec: Vec2) -> Vec2 {
        match self {
            Self::Down => Vec2::new(0.0, vec.y),
            Self::Right => Vec2::new(vec.x, 0.0),
        }
    }

    fn only_cross_axis(&self, vec: Vec2) -> Vec2 {
        match self {
            Self::Down => Vec2::new(vec.x, 0.0),
            Self::Right => Vec2::new(0.0, vec.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    x: f32,
    y: f32,
}

impl Alignment {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub const TOP_LEFT: Self = Self::new(0.0, 0.0);
    pub const TOP_CENTER: Self = Self::new(0.5, 0.0);
    pub const TOP_RIGHT: Self = Self::new(1.0, 0.0);

    pub const CENTER_LEFT: Self = Self::new(0.0, 0.5);
    pub const CENTER: Self = Self::new(0.5, 0.5);
    pub const CENTER_RIGHT: Self = Self::new(1.0, 0.5);

    pub const BOTTOM_LEFT: Self = Self::new(0.0, 1.0);
    pub const BOTTOM_CENTER: Self = Self::new(0.5, 1.0);
    pub const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);
}
