use std::borrow::Cow;

use {std::sync::Arc, yakui_core::paint::AddressMode};

use glam::UVec2;
use yakui_core::paint::{Texture, TextureFilter, TextureFormat};

pub(crate) struct GpuManagedTexture {
    size: UVec2,
    format: TextureFormat,
    gpu_texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub min_filter: wgpu::FilterMode,
    pub mag_filter: wgpu::FilterMode,
    pub address_mode: wgpu::AddressMode,
}

pub(crate) struct GpuTexture {
    pub view: Arc<wgpu::TextureView>,
    pub min_filter: wgpu::FilterMode,
    pub mag_filter: wgpu::FilterMode,
    pub mipmap_filter: wgpu::FilterMode,
    pub address_mode: wgpu::AddressMode,
}

impl GpuManagedTexture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture: &Texture) -> Self {
        let texture = premultiply_alpha(texture);

        let size = wgpu::Extent3d {
            width: texture.size().x,
            height: texture.size().y,
            depth_or_array_layers: 1,
        };

        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("yakui Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format(texture.format()),
            view_formats: &[],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture.data(),
            data_layout(texture.format(), texture.size()),
            size,
        );

        let gpu_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let min_filter = wgpu_filter_mode(texture.min_filter);
        let mag_filter = wgpu_filter_mode(texture.mag_filter);
        let address_mode = wgpu_address_mode(texture.address_mode);

        Self {
            size: texture.size(),
            format: texture.format(),
            gpu_texture,
            view: gpu_view,
            min_filter,
            mag_filter,
            address_mode,
        }
    }

    // Update the GpuTexture from a yakui Texture.
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &Texture) {
        if self.size != texture.size() || self.format != texture.format() {
            *self = Self::new(device, queue, texture);
            return;
        }

        let texture = premultiply_alpha(texture);

        let size = wgpu::Extent3d {
            width: texture.size().x,
            height: texture.size().y,
            depth_or_array_layers: 1,
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture.data(),
            data_layout(texture.format(), texture.size()),
            size,
        );
    }
}

fn data_layout(format: TextureFormat, size: UVec2) -> wgpu::ImageDataLayout {
    match format {
        TextureFormat::Rgba8Srgb => wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.x),
            rows_per_image: Some(size.y),
        },
        TextureFormat::R8 => wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size.x),
            rows_per_image: Some(size.y),
        },
        _ => panic!("Unsupported texture format {format:?}"),
    }
}

fn wgpu_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
        _ => panic!("Unsupported texture format {format:?}"),
    }
}

fn wgpu_filter_mode(filter: TextureFilter) -> wgpu::FilterMode {
    match filter {
        TextureFilter::Linear => wgpu::FilterMode::Linear,
        TextureFilter::Nearest => wgpu::FilterMode::Nearest,
    }
}

fn wgpu_address_mode(address_mode: AddressMode) -> wgpu::AddressMode {
    match address_mode {
        AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        AddressMode::Repeat => wgpu::AddressMode::Repeat,
    }
}

fn premultiply_alpha(texture: &Texture) -> Cow<'_, Texture> {
    fn premul(a: u8, b: u8) -> u8 {
        (((a as u32) * (b as u32) + 255) >> 8) as u8
    }

    match texture.format() {
        TextureFormat::Rgba8Srgb => {
            let mut texture = texture.clone();

            for pixel in texture.data_mut().chunks_exact_mut(4) {
                pixel[0] = premul(pixel[0], pixel[3]);
                pixel[1] = premul(pixel[1], pixel[3]);
                pixel[2] = premul(pixel[2], pixel[3]);
            }

            Cow::Owned(texture)
        }
        TextureFormat::R8 => Cow::Borrowed(texture),
        _ => Cow::Borrowed(texture),
    }
}
