use glam::Vec2;

/// Defines box constraints used for layout.
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Vec2,
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

    /// Returns the size closest to the given size that fits the constraints.
    pub fn constrain(&self, base: Vec2) -> Vec2 {
        base.max(self.min).min(self.max)
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
        self.max.is_finite()
    }

    /// Constraints are unbounded if the maximum size on either axis is
    /// infinite.
    pub fn is_unbounded(&self) -> bool {
        !self.is_bounded()
    }
}
