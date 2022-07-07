use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Vec2,
    pub max: Vec2,
}

impl Constraints {
    pub fn constrain(&self, base: Vec2) -> Vec2 {
        base.max(self.min).min(self.max)
    }
}
