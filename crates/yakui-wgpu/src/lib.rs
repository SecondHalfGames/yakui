#![allow(clippy::new_without_default)]

mod buffer;
mod texture;

use std::mem::size_of;
use std::ops::Range;

use buffer::Buffer;
use bytemuck::{Pod, Zeroable};
use glam::UVec2;
use thunderdome::Arena;
use yakui_core::paint::{Output, Texture, TextureFormat};
use yakui_core::{Vec2, Vec4};

use self::texture::GpuTexture;

pub struct State {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    default_texture: GpuTexture,
    default_sampler: wgpu::Sampler,
    textures: Arena<GpuTexture>,

    vertices: Buffer,
    indices: Buffer,
    commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    texcoord: Vec2,
    color: Vec4,
}

impl Vertex {
    const DESCRIPTOR: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
        ],
    };
}

impl State {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("yakui Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("yakui Render Pipeline Layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yakui Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESCRIPTOR],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let default_texture_data =
            Texture::new(TextureFormat::Rgba8Srgb, UVec2::new(1, 1), vec![255; 4]);
        let default_texture = GpuTexture::new(device, queue, &default_texture_data);

        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            layout,
            default_texture,
            default_sampler,
            textures: Arena::new(),

            vertices: Buffer::new(wgpu::BufferUsages::VERTEX),
            indices: Buffer::new(wgpu::BufferUsages::INDEX),
            commands: Vec::new(),
        }
    }

    pub fn paint(
        &mut self,
        state: &yakui_core::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_attachment: &wgpu::TextureView,
    ) -> wgpu::CommandBuffer {
        self.update_textures(state, device, queue);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("yakui Encoder"),
        });

        let output = state.paint();

        if output.meshes.is_empty() {
            return encoder.finish();
        }

        self.update_buffers(device, output);
        let vertices = self.vertices.upload(device, queue);
        let indices = self.indices.upload(device, queue);
        let commands = &self.commands;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_attachment,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, vertices.slice(..));
            render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

            for command in commands {
                render_pass.set_bind_group(0, &command.bind_group, &[]);
                render_pass.draw_indexed(command.index_range.clone(), 0, 0..1);
            }
        }

        encoder.finish()
    }

    fn update_buffers(&mut self, device: &wgpu::Device, output: Output) {
        self.vertices.clear();
        self.indices.clear();
        self.commands.clear();

        let commands = output.meshes.into_iter().map(|mesh| {
            let vertices = mesh.vertices.into_iter().map(|vertex| Vertex {
                pos: vertex.position,
                texcoord: vertex.texcoord,
                color: vertex.color,
            });

            let base = self.vertices.len() as u32;
            let indices = mesh.indices.into_iter().map(|index| base + index as u32);

            let start = self.indices.len() as u32;
            let end = start + indices.len() as u32;

            self.vertices.extend(vertices);
            self.indices.extend(indices);

            let texture = mesh
                .texture
                .and_then(|index| self.textures.get(index))
                .unwrap_or(&self.default_texture);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("yakui Bind Group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                    },
                ],
            });

            DrawCommand {
                index_range: start..end,
                bind_group,
            }
        });

        self.commands.extend(commands);
    }

    fn update_textures(
        &mut self,
        state: &yakui_core::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        for (index, texture) in state.textures() {
            match self.textures.get_mut(index) {
                Some(existing) => existing.update_if_newer(device, queue, texture),
                None => {
                    self.textures
                        .insert_at(index, GpuTexture::new(device, queue, texture));
                }
            }
        }
    }
}

struct DrawCommand {
    index_range: Range<u32>,
    bind_group: wgpu::BindGroup,
}
