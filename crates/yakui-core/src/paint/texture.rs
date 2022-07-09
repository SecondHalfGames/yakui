use glam::UVec2;

#[derive(Debug)]
pub struct Texture {
    format: TextureFormat,
    size: UVec2,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum TextureFormat {
    Rgba8,
}

impl Texture {
    pub fn new(format: TextureFormat, size: UVec2, data: Vec<u8>) -> Self {
        Self { format, size, data }
    }
}
