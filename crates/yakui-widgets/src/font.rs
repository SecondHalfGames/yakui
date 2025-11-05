use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

use crate::style::{FontFamily, FontKey, FontStyle, FontWeight, FontWidth};

#[derive(Clone)]
pub struct Fonts {
    inner: Rc<RefCell<FontsInner>>,
}

/// A 4-byte `OpenType` feature tag identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureTag([u8; 4]);

impl FeatureTag {
    pub const fn new(tag: &[u8; 4]) -> Self {
        Self(*tag)
    }

    /// Kerning adjusts spacing between specific character pairs
    pub const KERNING: Self = Self::new(b"kern");
    /// Standard ligatures (fi, fl, etc.)
    pub const STANDARD_LIGATURES: Self = Self::new(b"liga");
    /// Contextual ligatures (context-dependent ligatures)
    pub const CONTEXTUAL_LIGATURES: Self = Self::new(b"clig");
    /// Contextual alternates (glyph substitutions based on context)
    pub const CONTEXTUAL_ALTERNATES: Self = Self::new(b"calt");
    /// Discretionary ligatures (optional stylistic ligatures)
    pub const DISCRETIONARY_LIGATURES: Self = Self::new(b"dlig");
    /// Small caps (lowercase to small capitals)
    pub const SMALL_CAPS: Self = Self::new(b"smcp");
    /// All small caps (uppercase and lowercase to small capitals)
    pub const ALL_SMALL_CAPS: Self = Self::new(b"c2sc");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feature {
    pub tag: FeatureTag,
    pub value: u32,
}

impl Feature {
    pub const fn new(tag: &[u8; 4], value: u32) -> Self {
        Self {
            tag: FeatureTag::new(tag),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontAttributes {
    /// Allows you to set an explicit family name for the font, if the parsed name is wrong somehow.
    pub family_name: Option<Cow<'static, str>>,
    /// Letter spacing (tracking) in EM.
    pub letter_spacing: Option<f32>,
    /// Sets the [`OpenType` font features](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Fonts/OpenType_fonts).
    pub font_features: Vec<Feature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct FontKeyWithoutFamily {
    stretch: FontWidth,
    style: FontStyle,
    weight: FontWeight,
}

/// Storage for [`cosmic_text::AttrsOwned`] for widgets to borrow.
///
/// Note: the [`FontFamily`](crate::style::FontFamily) in the key will always be ['named'](crate::style::FontFamily::Name) in order to uniquely match the font.
///
/// The name will always be the first one in the list, which would be English US, unless it's missing. See also: [`cosmic_text::fontdb::FaceInfo`].
pub struct FontSelection {
    font_keys: BTreeMap<Cow<'static, str>, Vec<FontKeyWithoutFamily>>,
    cosmic_attrs: HashMap<FontKey, cosmic_text::AttrsOwned>,
    pub serif_family: Cow<'static, str>,
    pub sans_serif_family: Cow<'static, str>,
    pub cursive_family: Cow<'static, str>,
    pub fantasy_family: Cow<'static, str>,
    pub monospace_family: Cow<'static, str>,
}

impl FontSelection {
    fn new() -> Self {
        Self {
            font_keys: BTreeMap::new(),
            cosmic_attrs: HashMap::new(),
            serif_family: Cow::default(),
            sans_serif_family: Cow::default(),
            cursive_family: Cow::default(),
            fantasy_family: Cow::default(),
            monospace_family: Cow::default(),
        }
    }

    fn insert_cosmic_attrs(
        &mut self,
        face: &cosmic_text::fontdb::FaceInfo,
        font_attrs: &FontAttributes,
    ) {
        let Some(family_name) = font_attrs
            .family_name
            .clone()
            .or_else(|| face.families.first().map(|family| family.0.clone().into()))
        else {
            return;
        };

        if !face.monospaced && self.sans_serif_family.is_empty() {
            self.sans_serif_family = family_name.clone();
        } else if face.monospaced && self.monospace_family.is_empty() {
            self.monospace_family = family_name.clone();
        }

        let attrs = cosmic_text::AttrsOwned {
            stretch: face.stretch,
            style: face.style,
            weight: face.weight,

            family_owned: cosmic_text::FamilyOwned::Name(family_name.clone().into()),
            letter_spacing_opt: font_attrs.letter_spacing.map(cosmic_text::LetterSpacing),
            font_features: cosmic_text::FontFeatures {
                features: font_attrs
                    .font_features
                    .iter()
                    .map(|v| cosmic_text::Feature {
                        tag: cosmic_text::FeatureTag::new(&v.tag.0),
                        value: v.value,
                    })
                    .collect(),
            },

            // we don't care about these
            color_opt: None,
            metadata: 0,
            cache_key_flags: cosmic_text::CacheKeyFlags::empty(),
            metrics_opt: None,
        };

        let key = FontKey {
            family: FontFamily::Name(family_name.clone()),
            stretch: FontWidth::from(face.stretch),
            style: FontStyle::from(face.style),
            weight: FontWeight::from(face.weight),
        };

        self.font_keys
            .entry(family_name)
            .or_default()
            .push(FontKeyWithoutFamily {
                stretch: key.stretch,
                style: key.style,
                weight: key.weight,
            });

        if let Some(old) = self.cosmic_attrs.insert(key.clone(), attrs) {
            log::warn!("FontKey({key:?}) was already present with the following attributes:");
            log::warn!("{old:#?}");
        }
    }

    fn resolve_family_name(&self, family: &FontFamily) -> Cow<'static, str> {
        match family {
            FontFamily::Name(name) => name.clone(),
            FontFamily::Serif => self.serif_family.clone(),
            FontFamily::SansSerif => self.sans_serif_family.clone(),
            FontFamily::Cursive => self.cursive_family.clone(),
            FontFamily::Fantasy => self.fantasy_family.clone(),
            FontFamily::Monospace => self.monospace_family.clone(),
        }
    }

    fn query(&self, font_key: &FontKey) -> Option<FontKey> {
        let name = match &font_key.family {
            FontFamily::Name(v) => v,
            _ => return None,
        };

        let candidates = self.font_keys.get(name)?;
        if candidates.is_empty() {
            return None;
        }

        if let Some(index) = find_best_match(candidates.as_slice(), font_key) {
            let result = candidates[index];

            Some(FontKey {
                family: font_key.family.clone(),
                stretch: result.stretch,
                style: result.style,
                weight: result.weight,
            })
        } else {
            None
        }
    }

    pub fn get_cosmic_attrs(&self, font_key: &FontKey) -> cosmic_text::Attrs<'_> {
        let family_name = self.resolve_family_name(&font_key.family);

        self.query(&FontKey {
            family: FontFamily::Name(family_name.clone()),
            stretch: font_key.stretch,
            style: font_key.style,
            weight: font_key.weight,
        })
        .and_then(|font_key| self.cosmic_attrs.get(&font_key))
        .map(cosmic_text::AttrsOwned::as_attrs)
        .unwrap_or_else(|| {
            log::warn!(
                "Font family {:?} ({}) not found.",
                font_key.family,
                family_name
            );

            cosmic_text::Attrs::new()
        })
    }

    pub fn font_families(&self) -> impl Iterator<Item = &Cow<'static, str>> {
        self.font_keys.keys()
    }

    fn clear(&mut self) {
        self.cosmic_attrs.clear();
        self.font_keys.clear();
        self.serif_family = "".into();
        self.sans_serif_family = "".into();
        self.cursive_family = "".into();
        self.fantasy_family = "".into();
        self.monospace_family = "".into();
    }
}

pub struct FontsInner {
    pub font_system: cosmic_text::FontSystem,
    pub font_selection: FontSelection,
}

impl FontsInner {
    fn load_font(
        &mut self,
        source: cosmic_text::fontdb::Source,
        font_attrs: &FontAttributes,
    ) -> Vec<cosmic_text::fontdb::ID> {
        let ids = self.font_system.db_mut().load_font_source(source);
        for id in &ids {
            if let Some(face) = self.font_system.db().face(*id) {
                self.font_selection.insert_cosmic_attrs(face, font_attrs);
            }
        }
        ids.to_vec()
    }

    pub fn load_system_fonts(&mut self) {
        self.font_system.db_mut().load_system_fonts();

        let attrs = FontAttributes::default();
        for face in self.font_system.db().faces() {
            self.font_selection.insert_cosmic_attrs(face, &attrs);
        }

        let serif_family = self
            .font_system
            .db()
            .family_name(&cosmic_text::Family::Serif)
            .to_string();
        let sans_serif_family = self
            .font_system
            .db()
            .family_name(&cosmic_text::Family::SansSerif)
            .to_string();
        let cursive_family = self
            .font_system
            .db()
            .family_name(&cosmic_text::Family::Cursive)
            .to_string();
        let fantasy_family = self
            .font_system
            .db()
            .family_name(&cosmic_text::Family::Fantasy)
            .to_string();
        let monospace_family = self
            .font_system
            .db()
            .family_name(&cosmic_text::Family::Monospace)
            .to_string();

        if self.font_selection.serif_family.is_empty() && !serif_family.is_empty() {
            self.font_selection.serif_family = serif_family.into();
        }
        if self.font_selection.sans_serif_family.is_empty() && !sans_serif_family.is_empty() {
            self.font_selection.sans_serif_family = sans_serif_family.into();
        }
        if self.font_selection.cursive_family.is_empty() && !cursive_family.is_empty() {
            self.font_selection.cursive_family = cursive_family.into();
        }
        if self.font_selection.fantasy_family.is_empty() && !fantasy_family.is_empty() {
            self.font_selection.fantasy_family = fantasy_family.into();
        }
        if self.font_selection.monospace_family.is_empty() && !monospace_family.is_empty() {
            self.font_selection.monospace_family = monospace_family.into();
        }

        clear_family(self.font_system.db_mut());
    }

    pub fn clear_fonts(&mut self) {
        self.font_selection.clear();
        *self.font_system.db_mut() = cosmic_text::fontdb::Database::new();
        clear_family(self.font_system.db_mut());
    }
}

impl Fonts {
    fn new() -> Self {
        let font_system = cosmic_text::FontSystem::new_with_locale_and_db(
            sys_locale::get_locale().unwrap_or(String::from("en-US")),
            {
                let mut database = cosmic_text::fontdb::Database::new();
                clear_family(&mut database);
                database
            },
        );

        let mut inner = FontsInner {
            font_system,
            font_selection: FontSelection::new(),
        };

        #[cfg(feature = "default-fonts")]
        {
            static DEFAULT_BYTES: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

            inner.load_font(
                cosmic_text::fontdb::Source::Binary(Arc::from(&DEFAULT_BYTES)),
                &FontAttributes::default(),
            );
        }

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn inner(&self) -> Rc<RefCell<FontsInner>> {
        self.inner.clone()
    }

    pub fn with_inner<T>(&self, f: impl FnOnce(&mut FontsInner) -> T) -> T {
        let mut inner = self.inner.borrow_mut();

        f(&mut inner)
    }

    /// Loads the provided font.
    pub fn load_font_source(
        &self,
        source: cosmic_text::fontdb::Source,
    ) -> Vec<cosmic_text::fontdb::ID> {
        self.inner
            .borrow_mut()
            .load_font(source, &FontAttributes::default())
    }

    /// Loads the provided font, with the provided [`FontAttributes`].
    pub fn load_font_source_with_attrs(
        &self,
        source: cosmic_text::fontdb::Source,
        font_attrs: &FontAttributes,
    ) -> Vec<cosmic_text::fontdb::ID> {
        self.inner.borrow_mut().load_font(source, font_attrs)
    }

    /// Loads the fonts available on the system.
    pub fn load_system_fonts(&self) {
        self.inner.borrow_mut().load_system_fonts()
    }

    /// Sets the family that will be used by [`FontFamily::Serif`](crate::style::FontFamily::Serif).
    pub fn set_serif_family<S: Into<Cow<'static, str>>>(&self, family: S) {
        self.inner.borrow_mut().font_selection.serif_family = family.into()
    }

    /// Sets the family that will be used by [`FontFamily::SansSerif`](crate::style::FontFamily::SansSerif).
    pub fn set_sans_serif_family<S: Into<Cow<'static, str>>>(&self, family: S) {
        self.inner.borrow_mut().font_selection.sans_serif_family = family.into()
    }

    /// Sets the family that will be used by [`FontFamily::Cursive`](crate::style::FontFamily::Cursive).
    pub fn set_cursive_family<S: Into<Cow<'static, str>>>(&self, family: S) {
        self.inner.borrow_mut().font_selection.cursive_family = family.into()
    }

    /// Sets the family that will be used by [`FontFamily::Fantasy`](crate::style::FontFamily::Fantasy).
    pub fn set_fantasy_family<S: Into<Cow<'static, str>>>(&self, family: S) {
        self.inner.borrow_mut().font_selection.fantasy_family = family.into()
    }

    /// Sets the family that will be used by [`FontFamily::Monospace`](crate::style::FontFamily::Monospace).
    pub fn set_monospace_family<S: Into<Cow<'static, str>>>(&self, family: S) {
        self.inner.borrow_mut().font_selection.monospace_family = family.into()
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

// (copied from `fontdb`)
// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-style-matching
// Based on https://github.com/servo/font-kit
fn find_best_match(candidates: &[FontKeyWithoutFamily], font_key: &FontKey) -> Option<usize> {
    debug_assert!(!candidates.is_empty());

    // Step 4.
    let mut matching_set: Vec<usize> = (0..candidates.len()).collect();

    // Step 4a (`font-stretch`).
    let matches = matching_set
        .iter()
        .any(|&index| candidates[index].stretch == font_key.stretch);
    let matching_stretch = if matches {
        // Exact match.
        font_key.stretch
    } else if font_key.stretch <= FontWidth::Normal {
        // Closest stretch, first checking narrower values and then wider values.
        let stretch = matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch < font_key.stretch)
            .min_by_key(|&&index| {
                font_key.stretch.to_number() - candidates[index].stretch.to_number()
            });

        match stretch {
            Some(&matching_index) => candidates[matching_index].stretch,
            None => {
                let matching_index = *matching_set.iter().min_by_key(|&&index| {
                    candidates[index].stretch.to_number() - font_key.stretch.to_number()
                })?;

                candidates[matching_index].stretch
            }
        }
    } else {
        // Closest stretch, first checking wider values and then narrower values.
        let stretch = matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch > font_key.stretch)
            .min_by_key(|&&index| {
                candidates[index].stretch.to_number() - font_key.stretch.to_number()
            });

        match stretch {
            Some(&matching_index) => candidates[matching_index].stretch,
            None => {
                let matching_index = *matching_set.iter().min_by_key(|&&index| {
                    font_key.stretch.to_number() - candidates[index].stretch.to_number()
                })?;

                candidates[matching_index].stretch
            }
        }
    };
    matching_set.retain(|&index| candidates[index].stretch == matching_stretch);

    // Step 4b (`font-style`).
    let style_preference = match font_key.style {
        FontStyle::Italic => [FontStyle::Italic, FontStyle::Oblique, FontStyle::Normal],
        FontStyle::Oblique => [FontStyle::Oblique, FontStyle::Italic, FontStyle::Normal],
        FontStyle::Normal => [FontStyle::Normal, FontStyle::Oblique, FontStyle::Italic],
    };
    let matching_style = *style_preference.iter().find(|&query_style| {
        matching_set
            .iter()
            .any(|&index| candidates[index].style == *query_style)
    })?;

    matching_set.retain(|&index| candidates[index].style == matching_style);

    // Step 4c (`font-weight`).
    //
    // The spec doesn't say what to do if the weight is between 400 and 500 exclusive, so we
    // just use 450 as the cutoff.
    let weight = font_key.weight.0;

    let matching_weight = if matching_set
        .iter()
        .any(|&index| candidates[index].weight.0 == weight)
    {
        FontWeight(weight)
    } else if (400..450).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 500)
    {
        // Check 500 first.
        FontWeight::MEDIUM
    } else if (450..=500).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 400)
    {
        // Check 400 first.
        FontWeight::NORMAL
    } else if weight <= 500 {
        // Closest weight, first checking thinner values and then fatter ones.
        let idx = matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 <= weight)
            .min_by_key(|&&index| weight - candidates[index].weight.0);

        match idx {
            Some(&matching_index) => candidates[matching_index].weight,
            None => {
                let matching_index = *matching_set
                    .iter()
                    .min_by_key(|&&index| candidates[index].weight.0 - weight)?;
                candidates[matching_index].weight
            }
        }
    } else {
        // Closest weight, first checking fatter values and then thinner ones.
        let idx = matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 >= weight)
            .min_by_key(|&&index| candidates[index].weight.0 - weight);

        match idx {
            Some(&matching_index) => candidates[matching_index].weight,
            None => {
                let matching_index = *matching_set
                    .iter()
                    .min_by_key(|&&index| weight - candidates[index].weight.0)?;
                candidates[matching_index].weight
            }
        }
    };
    matching_set.retain(|&index| candidates[index].weight == matching_weight);

    // Ignore step 4d (`font-size`).

    // Return the result.
    matching_set.into_iter().next()
}

fn clear_family(database: &mut cosmic_text::fontdb::Database) {
    database.set_serif_family("");
    database.set_sans_serif_family("");
    database.set_cursive_family("");
    database.set_fantasy_family("");
    database.set_monospace_family("");
}
