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

    /// Create a new `Color` from a linear RGB color.
    pub fn from_linear(value: Vec4) -> Self {
        let linear = palette::LinSrgba::new(value.x, value.y, value.z, value.w);
        let (r, g, b, a) = palette::Srgba::from_linear(linear)
            .into_format::<u8, u8>()
            .into_components();

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
        palette::Srgba::new(self.r, self.g, self.b, self.a)
            .into_format::<f32, f32>()
            .into_linear()
            .into_components()
            .into()
    }
}

macro_rules! builtin_colors {
    ($($name:ident ($($color:literal),*),)*) => {
        #[allow(missing_docs)]
        impl Color {
            $(
                pub const $name: Self = Self::rgb($($color),*);
            )*
        }
    }
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
