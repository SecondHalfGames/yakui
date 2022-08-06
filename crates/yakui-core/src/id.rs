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

/// Identifies a texture that has been given to yakui to manage.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextureId(u64);

impl TextureId {
    /// Creates a new [TextureId].
    ///
    /// This is intended as a thin wrapper around a u64, which can mean whatever users want it to mean.
    #[inline]
    pub const fn new(index: u64) -> Self {
        Self(index)
    }

    /// Unwraps a [TextureId] into a `u64`.
    #[inline]
    pub fn inner(self) -> u64 {
        self.0
    }
}
