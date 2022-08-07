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

// #[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// /// Either a yakui or an external texture -- this is given for user ease.
// pub enum TextureId2 {
//     /// A yakui originating texture.[PaintDom] has the HashMap to convert these to
//     /// external textures.
//     Yakui(YakuiTexture),
//     /// An external originating texture.
//     External(ExternalTexture),
// }

// impl From<YakuiTexture> for TextureId2 {
//     fn from(o: YakuiTexture) -> Self {
//         Self::Yakui(o)
//     }
// }

// impl From<ExternalTexture> for TextureId2 {
//     fn from(o: ExternalTexture) -> Self {
//         Self::External(o)
//     }
// }

// /// A yakui originating texture.[PaintDom] has the HashMap to convert these to
// /// external textures.
// #[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(transparent)]
// pub struct YakuiTexture(u64);

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

// /// Identifies a texture that has been given to yakui to manage.
// #[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(transparent)]
// pub struct TextureId(u64);
