use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use image::RgbaImage;

use self::graphics::Graphics;

pub extern crate image;

mod graphics;

pub fn paint_and_save_to<P: AsRef<Path>>(state: &mut yakui_core::Yakui, path: P) {
    let path = path.as_ref();
    create_dir_all(path.parent().unwrap()).unwrap();

    let image = paint(state);
    let mut file = BufWriter::new(File::create(path).unwrap());
    image
        .write_to(&mut file, image::ImageOutputFormat::Png)
        .unwrap();
    file.flush().unwrap();
}

pub fn paint(state: &mut yakui_core::Yakui) -> RgbaImage {
    let graphics = pollster::block_on(Graphics::new());
    let mut renderer = yakui_wgpu::YakuiWgpu::new(&graphics.device, &graphics.queue);
    graphics.paint(state, &mut renderer)
}
