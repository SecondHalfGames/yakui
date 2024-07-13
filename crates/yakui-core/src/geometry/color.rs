use glam::Vec4;

/// An sRGB color with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// All black with no alpha
    pub const CLEAR: Self = Self::rgba(0, 0, 0, 0);

    /// Create a new `Color`.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new `Color` with full opacity.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color from a number, intended to be written as a hex literal.
    ///
    /// ```rust
    /// # use yakui_core::geometry::Color;
    /// let orange = Color::hex(0xff6137);
    /// assert_eq!(orange, Color::rgb(255, 97, 55));
    /// ```
    pub const fn hex(value: u32) -> Self {
        let r = ((value >> 16) & 255) as u8;
        let g = ((value >> 8) & 255) as u8;
        let b = (value & 255) as u8;

        Self { r, g, b, a: 255 }
    }

    /// Creates a new color using the existing color and the given linear alpha
    /// value given as a float.
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = (alpha * 255.0).round() as u8;
        self
    }

    /// Create a new `Color` from a linear RGB color.
    pub fn from_linear(value: Vec4) -> Self {
        let r = fast_srgb8::f32_to_srgb8(value.x);
        let g = fast_srgb8::f32_to_srgb8(value.y);
        let b = fast_srgb8::f32_to_srgb8(value.z);
        let a = (value.w * 255.0).round() as u8;
        Self::rgba(r, g, b, a)
    }

    /// Brighten or darken the color by multiplying it by a factor.
    ///
    /// Values greater than 1 will lighten the color, values smaller than 1 will
    /// darken it.
    ///
    /// This operation is gamma-correct.
    pub fn adjust(&self, factor: f32) -> Self {
        let linear = self.to_linear();
        let color = linear.truncate() * factor;
        let adjusted = color.extend(linear.w);

        Self::from_linear(adjusted)
    }

    /// Convert this color to a linear RGB color.
    pub fn to_linear(&self) -> Vec4 {
        let r = fast_srgb8::srgb8_to_f32(self.r);
        let g = fast_srgb8::srgb8_to_f32(self.g);
        let b = fast_srgb8::srgb8_to_f32(self.b);
        let a = self.a as f32 / 255.0;
        Vec4::new(r, g, b, a)
    }

    /// Blend with `other` in linear space
    pub fn lerp(&self, other: &Color, ratio: f32) -> Self {
        Self::from_linear(self.to_linear().lerp(other.to_linear(), ratio))
    }
}

macro_rules! builtin_colors {
    ($($name:ident ($r:literal, $g:literal, $b:literal),)*) => {
        #[allow(missing_docs)]
        impl Color {
            $(
                pub const $name: Self = Self::rgb($r, $g, $b);
            )*
        }
    };
}

builtin_colors! {
    RED(255, 0, 0),
    GREEN(0, 255, 0),
    BLUE(0, 0, 255),
    YELLOW(255, 255, 0),
    CYAN(0, 255, 255),
    FUCHSIA(255, 0, 255),
    WHITE(255, 255, 255),
    GRAY(127, 127, 127),
    BLACK(0, 0, 0),

    CORNFLOWER_BLUE(100, 149, 237),
    REBECCA_PURPLE(102, 51, 153),
}

impl From<[u8; 3]> for Color {
    #[inline]
    fn from(value: [u8; 3]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: 255,
        }
    }
}

impl From<[u8; 4]> for Color {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}
