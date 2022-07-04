use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Option<Vec2>,
    pub max: Option<Vec2>,
}

impl Constraints {
    pub fn constrain(&self, mut base: Vec2) -> Vec2 {
        if let Some(min) = self.min {
            base = base.max(min);
        }

        if let Some(max) = self.max {
            base = base.min(max);
        }

        base
    }
}
