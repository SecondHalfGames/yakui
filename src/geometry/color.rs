use glam::{Vec3, Vec4};

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

    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    pub fn as_vec4(&self, alpha: f32) -> Vec4 {
        self.as_vec3().extend(alpha)
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

impl From<[f32; 3]> for Color3 {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        Self {
            r: (value[0] * 255.0) as u8,
            g: (value[1] * 255.0) as u8,
            b: (value[2] * 255.0) as u8,
        }
    }
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
