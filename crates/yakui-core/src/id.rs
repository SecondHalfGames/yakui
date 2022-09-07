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

/// Identifies a texture that may be managed by yakui or handled by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureId {
    /// A texture that is managed by yakui, like the built-in font atlas or
    /// icons referenced by widgets.
    Managed(ManagedTextureId),

    /// A texture that is managed by the user or renderer.
    User(u64),
}

impl From<ManagedTextureId> for TextureId {
    fn from(value: ManagedTextureId) -> Self {
        Self::Managed(value)
    }
}

/// Identifies a texture that is managed by yakui.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ManagedTextureId(Index);

impl ManagedTextureId {
    #[inline]
    pub(crate) fn new(index: Index) -> Self {
        Self(index)
    }

    #[inline]
    pub(crate) fn index(&self) -> Index {
        self.0
    }
}

impl fmt::Debug for ManagedTextureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextureId({}, {})", self.0.slot(), self.0.generation())
    }
}
