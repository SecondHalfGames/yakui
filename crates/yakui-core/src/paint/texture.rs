use glam::UVec2;

/// A texture that is managed by yakui.
#[derive(Debug)]
pub struct Texture {
    format: TextureFormat,
    size: UVec2,
    data: Vec<u8>,

    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: TextureFilter,

    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: TextureFilter,

    /// Generation attached to the texture to indicate that it has been
    /// completely invalidated and should be reuploaded.
    pub(super) generation: u8,
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

impl Texture {
    /// Create a new texture from its format, size, and data.
    pub fn new(format: TextureFormat, size: UVec2, data: Vec<u8>) -> Self {
        Self {
            format,
            size,
            data,
            min_filter: TextureFilter::Nearest,
            mag_filter: TextureFilter::Linear,
            generation: 0,
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

    /// The texture's generation. This is incremented every time the texture has
    /// potentially been modified.
    pub fn generation(&self) -> u8 {
        self.generation
    }
}
