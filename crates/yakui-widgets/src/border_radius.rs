#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl From<f32> for BorderRadius {
    fn from(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
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
    pub fn uniform(radius: f32) -> Self {
        Self::from(radius)
    }

    pub fn new(top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        Self { top_left, top_right, bottom_left, bottom_right }
    }

    pub fn top(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }

    pub fn bottom(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn left(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: 0.0,
        }
    }

    pub fn right(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: radius,
        }
    }
}
