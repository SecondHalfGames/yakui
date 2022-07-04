use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pos: Vec2,
    size: Vec2,
}

impl Rect {
    pub const ZERO: Self = Self {
        pos: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }
}
