use yakui_core::geometry::Color;

use crate::font::FontName;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TextStyle {
    pub font: FontName,
    pub font_size: f32,
    pub color: Color,
    pub align: TextAlignment,
}

impl TextStyle {
    pub fn label() -> Self {
        Self {
            color: Color::WHITE,
            font: FontName::new("default"),
            font_size: 14.0,
            align: TextAlignment::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Start,
    Center,
    End,
}
