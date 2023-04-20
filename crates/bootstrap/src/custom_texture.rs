const WIDTH: usize = 64;
const HEIGHT: usize = 64;
const BYTES_PER_ROW: usize = WIDTH * 4;

pub fn generate(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let mut data = vec![0; WIDTH * HEIGHT * 4];

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let value = ((x + y) % 16) as u8 * 16;

            let start = x * 4 + y * BYTES_PER_ROW;
            let end = start + 4;
            data[start..end].copy_from_slice(&[value, value, value, 255]);
        }
    }

    let size = wgpu::Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Custom Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        view_formats: &[],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(BYTES_PER_ROW as u32),
            rows_per_image: None,
        },
        size,
    );

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
