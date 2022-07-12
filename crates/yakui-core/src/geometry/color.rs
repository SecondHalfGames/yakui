use glam::Vec3;

/// An sRGB color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color3 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color3 {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn adjust(&self, percent: f32) -> Self {
        Self {
            r: (self.r as f32 * percent) as u8,
            g: (self.g as f32 * percent) as u8,
            b: (self.b as f32 * percent) as u8,
        }
    }

    pub fn to_linear(&self) -> Vec3 {
        palette::Srgb::new(self.r, self.g, self.b)
            .into_format::<f32>()
            .into_linear()
            .into_components()
            .into()
    }
}

macro_rules! builtin_colors {
    ($($name:ident ($($color:literal),*),)*) => {
        impl Color3 {
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

impl From<[u8; 3]> for Color3 {
    #[inline]
    fn from(value: [u8; 3]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
        }
    }
}
