use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use smol_str::SmolStr;
use thunderdome::{Arena, Index};

pub use fontdue::{Font, FontSettings};

#[derive(Clone)]
pub struct Fonts {
    inner: Rc<RefCell<FontsInner>>,
}

struct FontsInner {
    storage: Arena<Font>,
    by_name: HashMap<FontName, FontId>,
}

impl Fonts {
    #[allow(unused_mut, unused_assignments)]
    fn new() -> Self {
        let mut storage = Arena::new();
        let mut by_name = HashMap::new();

        #[cfg(feature = "default-fonts")]
        {
            static DEFAULT_BYTES: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

            let font = Font::from_bytes(DEFAULT_BYTES, FontSettings::default())
                .expect("failed to load built-in font");
            let id = FontId::new(storage.insert(font));
            by_name.insert(FontName::new("default"), id);
        }

        let inner = Rc::new(RefCell::new(FontsInner { storage, by_name }));
        Self { inner }
    }

    pub fn add<S: Into<FontName>>(&self, font: Font, name: Option<S>) -> FontId {
        let mut inner = self.inner.borrow_mut();

        let id = FontId::new(inner.storage.insert(font));
        if let Some(name) = name {
            inner.by_name.insert(name.into(), id);
        }

        id
    }

    pub fn get(&self, name: &FontName) -> Option<Ref<'_, Font>> {
        let inner = self.inner.borrow();

        let &id = inner.by_name.get(name)?;
        if inner.storage.contains(id.0) {
            Some(Ref::map(inner, |inner| inner.storage.get(id.0).unwrap()))
        } else {
            None
        }
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontName(SmolStr);

impl FontName {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self(name.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for FontName {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<&String> for FontName {
    fn from(value: &String) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for FontName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifies a font that has been loaded and can be used by yakui.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FontId(Index);

impl FontId {
    #[inline]
    pub(crate) fn new(index: Index) -> Self {
        Self(index)
    }
}

impl fmt::Debug for FontId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FontId({}, {})", self.0.slot(), self.0.generation())
    }
}
