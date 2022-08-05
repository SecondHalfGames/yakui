use glam::UVec2;

/// A texture that is managed by yakui.
pub struct Texture {
    format: TextureFormat,
    size: UVec2,
    data: Vec<u8>,
    pub(super) generation: u8,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("format", &self.format)
            .field("size", &self.size)
            .field("generation", &self.generation)
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

impl Texture {
    /// Create a new texture from its format, size, and data.
    pub fn new(format: TextureFormat, size: UVec2, data: Vec<u8>) -> Self {
        Self {
            format,
            size,
            data,
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
