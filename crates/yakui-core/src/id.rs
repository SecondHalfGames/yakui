use std::fmt;

use thunderdome::Index;

/// Identifies a widget in the yakui DOM.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct WidgetId(Index);

impl WidgetId {
    #[inline]
    pub(crate) fn new(index: Index) -> Self {
        Self(index)
    }

    #[inline]
    pub(crate) fn index(&self) -> Index {
        self.0
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WidgetId({}, {})", self.0.slot(), self.0.generation())
    }
}

/// Identifies a yakui texture.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureId {
    /// This is a user submitted texture. Yakui treats these as opaque u64s
    User(u64),
    /// This is a yakui originating texture.
    Yak(Index),
}
