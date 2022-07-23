use yakui_core::geometry::Color3;

use crate::font::FontName;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color3,
    pub font: FontName,
    pub font_size: f32,
}

impl TextStyle {
    pub fn label() -> Self {
        Self {
            color: Color3::WHITE,
            font: FontName::new("default"),
            font_size: 14.0,
        }
    }
}
