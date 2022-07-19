//! Types used by various yakui widgets.

use crate::geometry::{Constraints, Vec2};

/// Defines sizing along a container's main axis.
///
/// For example, a horizontal list's main axis is horizontal, and a vertical
/// list's main axis is vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainAxisSize {
    /// Make the container fill all available space along its main axis.
    Max,

    /// Make the container fill the minimum amount of space along its main axis.
    Min,
}

/// Defines alignment along a container's main axis.
///
/// For example, a horizontal list's main axis is horizontal, and a vertical
/// list's main axis is vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainAxisAlignment {
    /// Align items to the beginning of the container's main axis.
    ///
    /// For a left-to-right list, this is the left side of the container.
    ///
    /// For a top-down list, this is the top of the container.
    Start,
    // Center,
    // End,
    // SpaceAround,
    // SpaceBetween,
    // SpaceEvenly,
}

/// Defines alignment along a container's cross axis.
///
/// For example, a horizontal list's cross axis is vertical, and a vertical
/// list's cross axis is horizontal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CrossAxisAlignment {
    /// Align items to the beginning of the container's cross axis.
    ///
    /// For a left-to-right list, this is the top of the container.
    ///
    /// For a top-down list, this is the left side of the container.
    Start,

    /// Align items to the center of the container's cross axis.
    Center,

    /// Align items to the end of the container's cross axis.
    ///
    /// For a left-to-right list, this is the bottom of the container.
    ///
    /// For a top-down list, this is the right side of the container.
    End,

    /// Stretch items to fill the maximum size of the container's cross axis.
    Stretch,
}

/// Defines the direction that a container will lay out its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Direction {
    /// Lay out children from top to bottom.
    Down,

    /// Lay out children from left to right.
    Right,
}

impl Direction {
    /// Constructs a [`Vec2`] with the given main and cross axis values.
    pub fn vec2(&self, main: f32, cross: f32) -> Vec2 {
        match self {
            Self::Down => Vec2::new(cross, main),
            Self::Right => Vec2::new(main, cross),
        }
    }

    /// Returns the main axis value from a [`Vec2`].
    pub fn get_main_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.y,
            Self::Right => vec.x,
        }
    }

    /// Returns the cross axis value from a [`Vec2`].
    pub fn get_cross_axis(&self, vec: Vec2) -> f32 {
        match self {
            Self::Down => vec.x,
            Self::Right => vec.y,
        }
    }

    /// Filters the [`Vec2`] to only contain its main axis value, setting the
    /// cross axis value to zero.
    pub fn only_main_axis(&self, vec: Vec2) -> Vec2 {
        match self {
            Self::Down => Vec2::new(0.0, vec.y),
            Self::Right => Vec2::new(vec.x, 0.0),
        }
    }

    /// Constrains a value to fit within the cross-axis limits defined by the
    /// constraints.
    pub fn constrain_cross_axis(&self, constraints: Constraints, value: f32) -> f32 {
        match self {
            Self::Down => constraints.constrain_width(value),
            Self::Right => constraints.constrain_height(value),
        }
    }
}

/// Defines alignment within a container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    x: f32,
    y: f32,
}

impl Alignment {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the anchor point for an alignment value.
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[allow(missing_docs)]
impl Alignment {
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
