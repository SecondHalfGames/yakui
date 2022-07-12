use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Vec2,
    pub max: Vec2,
}

impl Constraints {
    pub fn loose(max: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max,
        }
    }

    pub fn constrain(&self, base: Vec2) -> Vec2 {
        base.max(self.min).min(self.max)
    }

    pub fn is_loose(&self) -> bool {
        self.min == Vec2::ZERO
    }

    pub fn is_tight(&self) -> bool {
        self.min == self.max
    }

    pub fn is_bounded(&self) -> bool {
        self.max.is_finite()
    }

    pub fn is_unbounded(&self) -> bool {
        !self.is_bounded()
    }
}
