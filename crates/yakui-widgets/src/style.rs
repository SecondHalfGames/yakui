use std::borrow::Cow;

use yakui_core::geometry::Color;

use crate::auto_builders;

/// Specifies the width of the font.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontWidth {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontWidth {
    /// Returns a numeric representation of a width.
    #[inline]
    pub fn to_number(self) -> u16 {
        match self {
            FontWidth::UltraCondensed => 1,
            FontWidth::ExtraCondensed => 2,
            FontWidth::Condensed => 3,
            FontWidth::SemiCondensed => 4,
            FontWidth::Normal => 5,
            FontWidth::SemiExpanded => 6,
            FontWidth::Expanded => 7,
            FontWidth::ExtraExpanded => 8,
            FontWidth::UltraExpanded => 9,
        }
    }
}

impl From<cosmic_text::Stretch> for FontWidth {
    fn from(value: cosmic_text::Stretch) -> Self {
        match value {
            cosmic_text::Stretch::UltraCondensed => Self::UltraCondensed,
            cosmic_text::Stretch::ExtraCondensed => Self::ExtraCondensed,
            cosmic_text::Stretch::Condensed => Self::Condensed,
            cosmic_text::Stretch::SemiCondensed => Self::SemiCondensed,
            cosmic_text::Stretch::Normal => Self::Normal,
            cosmic_text::Stretch::SemiExpanded => Self::SemiExpanded,
            cosmic_text::Stretch::Expanded => Self::Expanded,
            cosmic_text::Stretch::ExtraExpanded => Self::ExtraExpanded,
            cosmic_text::Stretch::UltraExpanded => Self::UltraExpanded,
        }
    }
}

/// Specifies the style of the font (i.e. italic, oblique).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// A face that is neither italic not obliqued.
    #[default]
    Normal,
    /// A form that is generally cursive in nature.
    Italic,
    /// A typically-sloped version of the regular face.
    Oblique,
}

impl From<cosmic_text::Style> for FontStyle {
    fn from(value: cosmic_text::Style) -> Self {
        match value {
            cosmic_text::Style::Normal => Self::Normal,
            cosmic_text::Style::Italic => Self::Italic,
            cosmic_text::Style::Oblique => Self::Oblique,
        }
    }
}

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontWeight(pub u16);

impl Default for FontWeight {
    #[inline]
    fn default() -> FontWeight {
        FontWeight::NORMAL
    }
}

impl FontWeight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: FontWeight = FontWeight(100);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    /// Light weight (300).
    pub const LIGHT: FontWeight = FontWeight(300);
    /// Normal (400).
    pub const NORMAL: FontWeight = FontWeight(400);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: FontWeight = FontWeight(500);
    /// Semibold weight (600).
    pub const SEMIBOLD: FontWeight = FontWeight(600);
    /// Bold weight (700).
    pub const BOLD: FontWeight = FontWeight(700);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    /// Black weight (900), the thickest value.
    pub const BLACK: FontWeight = FontWeight(900);
}

impl From<cosmic_text::Weight> for FontWeight {
    fn from(value: cosmic_text::Weight) -> Self {
        Self(value.0)
    }
}

/// Specifies the family of the font.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// The font with the specified name.
    Name(Cow<'static, str>),
    Serif,
    #[default]
    SansSerif,
    Cursive,
    Fantasy,
    Monospace,
}

impl<'a> From<cosmic_text::Family<'a>> for FontFamily {
    fn from(value: cosmic_text::Family) -> Self {
        match value {
            cosmic_text::Family::Name(s) => Self::Name(s.to_string().into()),
            cosmic_text::Family::Serif => Self::Serif,
            cosmic_text::Family::SansSerif => Self::SansSerif,
            cosmic_text::Family::Cursive => Self::Cursive,
            cosmic_text::Family::Fantasy => Self::Fantasy,
            cosmic_text::Family::Monospace => Self::Monospace,
        }
    }
}

/// Specifies which font to be used for a piece of text.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FontKey {
    pub family: FontFamily,
    pub stretch: FontWidth,
    pub style: FontStyle,
    pub weight: FontWeight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub font: FontKey,
    pub line_height_override: Option<f32>,
    pub color: Color,
    pub align: TextAlignment,
}

auto_builders!(TextStyle {
    font_size: f32,
    font: FontKey,
    line_height_override: Option<f32>,
    color: Color,
    align: TextAlignment,
});

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font: FontKey::default(),
            line_height_override: None,
            color: Color::WHITE,
            align: TextAlignment::Start,
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
        self.line_height_override.unwrap_or(self.font_size * 1.2)
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

impl From<TextAlignment> for Option<cosmic_text::Align> {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Start => None,
            TextAlignment::Center => Some(cosmic_text::Align::Center),
            TextAlignment::End => Some(cosmic_text::Align::End),
        }
    }
}
