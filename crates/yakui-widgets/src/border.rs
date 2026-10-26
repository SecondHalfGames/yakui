use yakui_core::geometry::Color;

use crate::auto_builders;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    pub color: Color,
    pub width: f32,
}

impl Border {
    pub const fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }
}

impl From<Color> for Border {
    fn from(color: Color) -> Self {
        Self { color, width: 1.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

auto_builders!(BorderRadius {
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
});

impl From<f32> for BorderRadius {
    fn from(radius: f32) -> Self {
        Self::uniform(radius)
    }
}

impl From<(f32, f32, f32, f32)> for BorderRadius {
    fn from((top_left, top_right, bottom_left, bottom_right): (f32, f32, f32, f32)) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
}

impl BorderRadius {
    pub const ZERO: Self = Self::uniform(0.0);

    pub const fn none() -> Self {
        Self::ZERO
    }

    pub const fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub const fn new(top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub const fn top(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }

    pub const fn bottom(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub const fn left(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: 0.0,
        }
    }

    pub const fn right(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: radius,
        }
    }
}
