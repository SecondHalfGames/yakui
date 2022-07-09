use glam::UVec2;

#[derive(Debug)]
pub struct Texture {
    format: TextureFormat,
    size: UVec2,
    data: Vec<u8>,
    pub(super) generation: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextureFormat {
    Rgba8Srgb,
    R8,
}

impl Texture {
    pub fn new(format: TextureFormat, size: UVec2, data: Vec<u8>) -> Self {
        Self {
            format,
            size,
            data,
            generation: 0,
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn generation(&self) -> u8 {
        self.generation
    }
}
