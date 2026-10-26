use crate::samplers::Samplers;

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
    pub mipmap_filter: wgpu::MipmapFilterMode,
    pub address_mode: wgpu::AddressMode,
}

impl GpuManagedTexture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
        premul_pipeline: &wgpu::RenderPipeline,
        premul_bind_group_layout: &wgpu::BindGroupLayout,
        samplers: &Samplers,
    ) -> Self {
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

        if matches!(texture.format(), TextureFormat::Rgba8Srgb) {
            premultiply_alpha(
                device,
                queue,
                texture,
                &gpu_texture,
                size,
                premul_pipeline,
                premul_bind_group_layout,
                samplers,
            );
        } else {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &gpu_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                texture.data(),
                data_layout(texture.format(), texture.size()),
                size,
            );
        }

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
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
        premul_pipeline: &wgpu::RenderPipeline,
        premul_bind_group_layout: &wgpu::BindGroupLayout,
        samplers: &Samplers,
    ) {
        if self.size != texture.size() || self.format != texture.format() {
            *self = Self::new(
                device,
                queue,
                texture,
                premul_pipeline,
                premul_bind_group_layout,
                samplers,
            );
            return;
        }

        let size = wgpu::Extent3d {
            width: texture.size().x,
            height: texture.size().y,
            depth_or_array_layers: 1,
        };

        if matches!(texture.format(), TextureFormat::Rgba8Srgb) {
            premultiply_alpha(
                device,
                queue,
                texture,
                &self.gpu_texture,
                size,
                premul_pipeline,
                premul_bind_group_layout,
                samplers,
            );
        } else {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
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
}

fn data_layout(format: TextureFormat, size: UVec2) -> wgpu::TexelCopyBufferLayout {
    match format {
        TextureFormat::Rgba8Srgb => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.x),
            rows_per_image: Some(size.y),
        },
        TextureFormat::Rgba8SrgbPremultiplied => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.x),
            rows_per_image: Some(size.y),
        },
        TextureFormat::R8 => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.x),
            rows_per_image: Some(size.y),
        },
    }
}

fn wgpu_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba8SrgbPremultiplied => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
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

#[allow(clippy::too_many_arguments)]
fn premultiply_alpha(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &Texture,
    gpu_texture: &wgpu::Texture,
    size: wgpu::Extent3d,
    premul_pipeline: &wgpu::RenderPipeline,
    premul_bind_group_layout: &wgpu::BindGroupLayout,
    samplers: &Samplers,
) {
    let source_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Premultiply Source Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu_format(texture.format()),
        view_formats: &[],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    let destination_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Premultiply Destination Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu_format(texture.format()),
        view_formats: &[],
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &source_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        texture.data(),
        data_layout(texture.format(), texture.size()),
        size,
    );

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Premultiply Texture Bind Group"),
        layout: premul_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &source_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(samplers.get(
                    wgpu::FilterMode::Nearest,
                    wgpu::FilterMode::Nearest,
                    wgpu::MipmapFilterMode::Nearest,
                    wgpu::AddressMode::ClampToEdge,
                )),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Premultiply Texture Encoder"),
    });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("yakui Premultiply Texture Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &destination_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    // SAFETY: we'll be reading every pixel of the source texture and we don't care the content of the newly created destination texture.
                    // however, if the source texture is corrupt, then this will be too.
                    load: wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() }),
                    store: wgpu::StoreOp::default(),
                },
            })],
            ..Default::default()
        });

        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_pipeline(premul_pipeline);
        render_pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &destination_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: gpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        size,
    );

    queue.submit([encoder.finish()]);
}
