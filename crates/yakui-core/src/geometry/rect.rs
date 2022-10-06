use glam::Vec2;

/// A bounding rectangle with floating point coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pos: Vec2,
    size: Vec2,
}

impl Rect {
    /// A zero-sized rectangle at the origin.
    pub const ZERO: Self = Self {
        pos: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    /// A rectangle of size (1, 1) at the origin.
    pub const ONE: Self = Self {
        pos: Vec2::ZERO,
        size: Vec2::ONE,
    };

    /// Create a `Rect` from a position and size.
    #[inline]
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    /// The position of the rectangle's upper-left corner. This is the minimum
    /// value enclosed by the rectangle.
    #[inline]
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    /// The size of the rectangle.
    #[inline]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// The maximum value enclosed by the rectangle.
    #[inline]
    pub fn max(&self) -> Vec2 {
        self.pos + self.size
    }

    /// Set the rectangle's position.
    #[inline]
    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    /// Set the rectangle's size.
    #[inline]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    /// Set the rectangle's maximum extent.
    #[inline]
    pub fn set_max(&mut self, max: Vec2) {
        self.size = max - self.pos;
    }

    /// Tells whether the given point is contained within the rectangle.
    ///
    /// If the point lies on the rectangle's boundary, it is considered
    /// **inside**.
    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
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

    /// Scale the rectangle by dividing it by a vector.
    #[inline]
    pub fn div_vec2(&self, size: Vec2) -> Self {
        Self::from_pos_size(self.pos / size, self.size / size)
    }

    /// Returns a rectangle that fits this rectangle and the given rectangle.
    #[inline]
    pub fn constrain(mut self, other: Rect) -> Self {
        let min = self.pos().max(other.pos());
        let max = self.max().min(other.max());

        self.set_pos(min);
        self.set_max(max);
        self
    }
}
