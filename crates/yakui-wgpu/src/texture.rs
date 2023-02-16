use std::num::NonZeroU32;

use glam::UVec2;
use yakui_core::paint::{Texture, TextureFilter, TextureFormat};

pub(crate) struct GpuManagedTexture {
    size: UVec2,
    format: TextureFormat,
    gpu_texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub min_filter: wgpu::FilterMode,
    pub mag_filter: wgpu::FilterMode,
}

pub(crate) struct GpuTexture {
    pub view: wgpu::TextureView,
    pub min_filter: wgpu::FilterMode,
    pub mag_filter: wgpu::FilterMode,
}

impl GpuManagedTexture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture: &Texture) -> Self {
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

        Self {
            size: texture.size(),
            format: texture.format(),
            gpu_texture,
            view: gpu_view,
            min_filter,
            mag_filter,
        }
    }

    // Update the GpuTexture from a yakui Texture.
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &Texture) {
        if self.size != texture.size() || self.format != texture.format() {
            *self = Self::new(device, queue, texture);
            return;
        }

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
            bytes_per_row: NonZeroU32::new(4 * size.x),
            rows_per_image: NonZeroU32::new(size.y),
        },
        TextureFormat::R8 => wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(size.x),
            rows_per_image: NonZeroU32::new(size.y),
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
