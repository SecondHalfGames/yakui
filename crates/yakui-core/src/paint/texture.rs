use glam::UVec2;

/// A texture that is managed by yakui.
#[derive(Clone)]
pub struct Texture {
    format: TextureFormat,
    size: UVec2,
    data: Vec<u8>,

    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: TextureFilter,

    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: TextureFilter,

    /// How to handle texture addressing
    pub address_mode: AddressMode,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("format", &self.format)
            .field("size", &self.size)
            .field("min_filter", &self.min_filter)
            .field("mag_filter", &self.mag_filter)
            .field("address_mode", &self.address_mode)
            .finish_non_exhaustive()
    }
}

/// A texture format that yakui can manage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextureFormat {
    /// Red, green, blue, and alpha channels, each represented as a `u8`. The
    /// color channels are sRGB-encoded.
    Rgba8Srgb,

    /// A single color channel represented as a `u8`.
    R8,
}

/// Which kind of filtering to use when scaling the texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    /// Blend the nearest pixels in the texture.
    Linear,

    /// Pick the nearest pixel. Useful for pixel art.
    Nearest,
}

/// Which kind of address mode to use when UVs go outside the range `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressMode {
    /// Clamp to the edge of the texture
    ClampToEdge,

    /// Repeat the texture
    Repeat,
}

impl Texture {
    /// Create a new texture from its format, size, and data.
    pub fn new(format: TextureFormat, size: UVec2, data: Vec<u8>) -> Self {
        Self {
            format,
            size,
            data,
            min_filter: TextureFilter::Nearest,
            mag_filter: TextureFilter::Linear,
            address_mode: AddressMode::ClampToEdge,
        }
    }

    /// The size of the texture.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// The texture's raw data.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// A mutable reference to the texture's data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// The texture's format.
    pub fn format(&self) -> TextureFormat {
        self.format
    }
}

/// Describes a change that happened to a texture since the last update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureChange {
    /// The texture was added since the last update.
    Added,

    /// The texture was removed since the last update.
    Removed,

    /// The texture was modified since the last update.
    Modified,
}
