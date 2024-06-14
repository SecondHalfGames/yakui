use glam::Vec2;

/// Defines box constraints used for layout.
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    /// The minimum size that is allowed by these constraints.
    pub min: Vec2,

    /// The maximum size that is allowed by these constraints.
    pub max: Vec2,
}

impl Constraints {
    /// Create a new `Constraints` with a minimum size of zero and the given
    /// maximum.
    pub fn loose(max: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max,
        }
    }

    /// Create a new `Constraints` whose minimum and maximum constraints are the
    /// given value.
    pub fn tight(value: Vec2) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    /// Create a new `Constraints` whose minimum size is zero and whose maximum
    /// is infinite.
    pub fn none() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::new(f32::INFINITY, f32::INFINITY),
        }
    }

    /// Returns the size closest to the given size that satisfies the minimun
    /// constraints.
    pub fn constrain_min(&self, base: Vec2) -> Vec2 {
        base.max(self.min)
    }

    /// Returns the size closest to the given size that fits the constraints.
    pub fn constrain(&self, base: Vec2) -> Vec2 {
        base.max(self.min).min(self.max)
    }

    /// Returns the width closest to the given width that fits the constraints.
    pub fn constrain_width(&self, width: f32) -> f32 {
        width.max(self.min.x).min(self.max.x)
    }

    /// Returns the height closest to the given height that fits the constraints.
    pub fn constrain_height(&self, height: f32) -> f32 {
        height.max(self.min.y).min(self.max.y)
    }

    /// Constraints are loose if there is no minimum size.
    pub fn is_loose(&self) -> bool {
        self.min == Vec2::ZERO
    }

    /// Constraints are tight if the minimum size and maximum size are the same.
    /// This means that there is exactly only size that satisfies the
    /// constraints.
    pub fn is_tight(&self) -> bool {
        self.min == self.max
    }

    /// Constraints are bounded if the maximum size on both axes is finite.
    pub fn is_bounded(&self) -> bool {
        self.max.is_finite() && is_nonmax(self.max)
    }

    /// Constraints are unbounded if the maximum size on either axis is
    /// infinite.
    pub fn is_unbounded(&self) -> bool {
        !self.is_bounded()
    }
}

fn is_nonmax(value: Vec2) -> bool {
    value.x < f32::MAX && value.y < f32::MAX
}
