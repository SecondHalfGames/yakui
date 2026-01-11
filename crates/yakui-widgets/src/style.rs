use yakui_core::geometry::Color;

use crate::auto_builders;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height_override: Option<f32>,
    pub color: Color,
    pub align: TextAlignment,
    pub attrs: cosmic_text::AttrsOwned,
}

auto_builders!(TextStyle {
    font_size: f32,
    line_height_override: Option<f32>,
    color: Color,
    align: TextAlignment,
    attrs: cosmic_text::AttrsOwned,
});

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height_override: None,
            color: Color::WHITE,
            align: TextAlignment::Start,
            attrs: cosmic_text::AttrsOwned {
                family_owned: cosmic_text::FamilyOwned::SansSerif,
                ..cosmic_text::AttrsOwned::new(&cosmic_text::Attrs::new())
            },
        }
    }
}

impl TextStyle {
    pub fn label() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn line_height(&self) -> f32 {
        self.line_height_override.unwrap_or(self.font_size * 1.175)
    }

    pub fn to_metrics(&self, scale_factor: f32) -> cosmic_text::Metrics {
        cosmic_text::Metrics::new(
            (self.font_size * scale_factor).ceil(),
            (self.line_height() * scale_factor).ceil(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Start,
    Center,
    End,
}

impl From<TextAlignment> for cosmic_text::Align {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Start => cosmic_text::Align::Left,
            TextAlignment::Center => cosmic_text::Align::Center,
            TextAlignment::End => cosmic_text::Align::Right,
        }
    }
}
