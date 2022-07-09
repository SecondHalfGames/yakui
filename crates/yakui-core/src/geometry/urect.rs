use glam::UVec2;

use super::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct URect {
    pos: UVec2,
    size: UVec2,
}

impl URect {
    pub const ZERO: Self = Self {
        pos: UVec2::ZERO,
        size: UVec2::ZERO,
    };

    pub const ONE: Self = Self {
        pos: UVec2::ZERO,
        size: UVec2::ONE,
    };

    #[inline]
    pub fn from_pos_size(pos: UVec2, size: UVec2) -> Self {
        Self { pos, size }
    }

    #[inline]
    pub fn as_rect(&self) -> Rect {
        Rect::from_pos_size(self.pos.as_vec2(), self.size.as_vec2())
    }

    #[inline]
    pub fn pos(&self) -> UVec2 {
        self.pos
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        self.size
    }

    #[inline]
    pub fn max(&self) -> UVec2 {
        self.pos + self.size
    }

    #[inline]
    pub fn set_pos(&mut self, pos: UVec2) {
        self.pos = pos;
    }

    #[inline]
    pub fn set_size(&mut self, size: UVec2) {
        self.size = size;
    }

    #[inline]
    pub fn contains_point(&self, point: UVec2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let self_max = self.max();
        let other_max = other.max();

        let x_intersect = self.pos.x < other_max.x && self_max.x > other.pos.x;
        let y_intersect = self.pos.y < other_max.y && self_max.y > other.pos.y;

        x_intersect && y_intersect
    }
}
