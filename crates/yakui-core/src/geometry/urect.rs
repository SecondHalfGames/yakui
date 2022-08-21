use glam::UVec2;

use super::Rect;

/// A bounding rectangle with positive integer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct URect {
    pos: UVec2,
    size: UVec2,
}

impl URect {
    /// A zero-sized rectangle at the origin.
    pub const ZERO: Self = Self {
        pos: UVec2::ZERO,
        size: UVec2::ZERO,
    };

    /// A rectangle of size (1, 1) at the origin.
    pub const ONE: Self = Self {
        pos: UVec2::ZERO,
        size: UVec2::ONE,
    };

    /// Create a `Rect` from a position and size.
    #[inline]
    pub fn from_pos_size(pos: UVec2, size: UVec2) -> Self {
        Self { pos, size }
    }

    /// The position of the rectangle's upper-left corner. This is the minimum
    /// value enclosed by the rectangle.
    #[inline]
    pub fn as_rect(&self) -> Rect {
        Rect::from_pos_size(self.pos.as_vec2(), self.size.as_vec2())
    }

    /// The size of the rectangle.
    #[inline]
    pub fn pos(&self) -> UVec2 {
        self.pos
    }

    /// The size of the rectangle.
    #[inline]
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// The maximum value enclosed by the rectangle.
    #[inline]
    pub fn max(&self) -> UVec2 {
        self.pos + self.size
    }

    /// Set the rectangle's position.
    #[inline]
    pub fn set_pos(&mut self, pos: UVec2) {
        self.pos = pos;
    }

    /// Set the rectangle's size.
    #[inline]
    pub fn set_size(&mut self, size: UVec2) {
        self.size = size;
    }

    /// Tells whether the given point is contained within the rectangle.
    ///
    /// If the point lies on the rectangle's boundary, it is considered
    /// **inside**.
    #[inline]
    pub fn contains_point(&self, point: UVec2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }

    /// Tells whether two rectangles intersect.
    ///
    /// If the rectangles touch but do not overlap, they are considered **not
    /// intersecting**.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let self_max = self.max();
        let other_max = other.max();

        let x_intersect = self.pos.x < other_max.x && self_max.x > other.pos.x;
        let y_intersect = self.pos.y < other_max.y && self_max.y > other.pos.y;

        x_intersect && y_intersect
    }
}
