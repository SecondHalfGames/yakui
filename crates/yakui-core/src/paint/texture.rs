#[derive(Debug)]
pub struct Texture {
    format: TextureFormat,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
}

impl Texture {
    pub(crate) fn new(format: TextureFormat, data: Vec<u8>) -> Self {
        Self { format, data }
    }
}
