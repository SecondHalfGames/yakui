//! Types used by various yakui widgets.

use yakui_core::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainAxisSize {
    Max,
    // Min,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainAxisAlignment {
    Start,
    // Center,
    // End,
    // SpaceAround,
    // SpaceBetween,
    // SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CrossAxisAlignment {
    Start,
    // Center,
    // End,
    // Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Direction {
    Down,
    Right,
}

impl Direction {
    pub(crate) fn vec2(&self, main: f32, cross: f32) -> Vec2 {
        match self {
            Self::Down => Vec2::new(cross, main),
            Self::Right => Vec2::new(main, cross),
        }
    }

    pub(crate) fn get_main_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.y,
            Self::Right => vec.x,
        }
    }

    pub(crate) fn get_cross_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.x,
            Self::Right => vec.y,
        }
    }

    pub(crate) fn only_main_axis(&self, vec: Vec2) -> Vec2 {
        match self {
            Self::Down => Vec2::new(0.0, vec.y),
            Self::Right => Vec2::new(vec.x, 0.0),
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

    pub(crate) fn as_vec2(&self) -> Vec2 {
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
