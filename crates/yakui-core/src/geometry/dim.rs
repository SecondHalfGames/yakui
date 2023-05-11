use glam::Vec2;

/// Describes a length consisting of one or more measurements added together.
///
/// This struct is inspired by CSS's unit system, which is a very flexible way
/// to talk about the size of widgets in a layout.
///
/// The equivalent CSS for a given `Dim` is:
///
/// ```css
/// calc(dim.pixels + 100 * dim.percent)
/// ```
///
/// where `dim.pixels` is the `px` unit in CSS and `dim.percent` is the `%` unit
/// in CSS.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dim {
    /// The portion of the value in logical pixels. Works like the `px` unit in
    /// CSS.
    pub pixels: f32,

    /// A value scaled based on the parent object's measurement on the axis.
    /// `1.0` corresponds to 100% of the parent width, while `0.0` corresponds
    /// to 0%.
    pub percent: f32,
}

impl Dim {
    /// A `Dim` corresponding to a length of zero.
    pub const ZERO: Self = Self {
        pixels: 0.0,
        percent: 0.0,
    };

    /// Returns a `Dim` with the given length in pixels.
    pub const fn pixels(pixels: f32) -> Self {
        Self {
            pixels,
            ..Self::ZERO
        }
    }

    /// Returns a `Dim` with the given length as a percentage of the parent
    /// container.
    pub const fn percent(percent: f32) -> Self {
        Self {
            percent,
            ..Self::ZERO
        }
    }

    /// Resolves the `Dim` to a single value in pixels using information about
    /// the surrounding context.
    pub fn resolve(&self, parent_length: f32) -> f32 {
        self.pixels + parent_length * self.percent
    }
}

/// A size or position in 2D based on one or more measurements added together.
///
/// See [`Dim`] for more information on the units available and their meaning.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dim2 {
    /// The dimension on the X axis.
    pub x: Dim,

    /// The dimension on the Y axis.
    pub y: Dim,
}

impl Dim2 {
    /// A `Dim2` corresponding to zero on both axes.
    pub const ZERO: Self = Dim2 {
        x: Dim::ZERO,
        y: Dim::ZERO,
    };

    /// Create a new `Dim2` from two [`Dim`] values.
    pub const fn new(x: Dim, y: Dim) -> Self {
        Self { x, y }
    }

    /// Create a new `Dim2` with pixel values for each axis.
    pub const fn pixels(x: f32, y: f32) -> Self {
        Self {
            x: Dim::pixels(x),
            y: Dim::pixels(y),
        }
    }

    /// Resolves the `Dim2` to a measurement in pixels using information about
    /// the surrounding context.
    pub fn resolve(&self, parent_size: Vec2) -> Vec2 {
        let x = self.x.resolve(parent_size.x);
        let y = self.y.resolve(parent_size.y);

        Vec2::new(x, y)
    }
}
