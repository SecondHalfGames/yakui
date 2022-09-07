#![allow(clippy::new_without_default)]

mod buffer;
mod samplers;
mod texture;

use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;

use buffer::Buffer;
use bytemuck::{Pod, Zeroable};
use glam::UVec2;
use yakui_core::geometry::{Vec2, Vec4};
use yakui_core::paint::{PaintDom, Pipeline, Texture, TextureFormat};
use yakui_core::ManagedTextureId;

use self::samplers::Samplers;
use self::texture::GpuTexture;

pub struct State {
    main_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    default_texture: GpuTexture,
    samplers: Samplers,
    textures: HashMap<ManagedTextureId, GpuTexture>,

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
        let main_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
        });

        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/text.wgsl").into()),
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

        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yakui Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &main_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESCRIPTOR],
            },
            fragment: Some(wgpu::FragmentState {
                module: &main_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yakui Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESCRIPTOR],
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

        let samplers = Samplers::new(device);

        let default_texture_data =
            Texture::new(TextureFormat::Rgba8Srgb, UVec2::new(1, 1), vec![255; 4]);
        let default_texture = GpuTexture::new(device, queue, &default_texture_data);

        Self {
            main_pipeline,
            text_pipeline,
            layout,
            default_texture,
            samplers,
            textures: HashMap::new(),

            vertices: Buffer::new(wgpu::BufferUsages::VERTEX),
            indices: Buffer::new(wgpu::BufferUsages::INDEX),
            commands: Vec::new(),
        }
    }

    pub fn paint(
        &mut self,
        state: &mut yakui_core::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_attachment: &wgpu::TextureView,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("yakui Encoder"),
        });

        self.paint_with_encoder(state, device, queue, &mut encoder, color_attachment);

        encoder.finish()
    }

    pub fn paint_with_encoder(
        &mut self,
        state: &mut yakui_core::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_attachment: &wgpu::TextureView,
    ) {
        profiling::scope!("yakui-wgpu paint_with_encoder");

        let paint = state.paint();

        if paint.calls().is_empty() {
            return;
        }

        self.update_textures(device, paint, queue);
        self.update_buffers(device, paint);
        let vertices = self.vertices.upload(device, queue);
        let indices = self.indices.upload(device, queue);
        let commands = &self.commands;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("yakui Render Pass"),
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

            render_pass.set_vertex_buffer(0, vertices.slice(..));
            render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

            for command in commands {
                match command.pipeline {
                    Pipeline::Main => render_pass.set_pipeline(&self.main_pipeline),
                    Pipeline::Text => render_pass.set_pipeline(&self.text_pipeline),
                    _ => continue,
                }

                render_pass.set_bind_group(0, &command.bind_group, &[]);
                render_pass.draw_indexed(command.index_range.clone(), 0, 0..1);
            }
        }
    }

    fn update_buffers(&mut self, device: &wgpu::Device, paint: &PaintDom) {
        profiling::scope!("update_buffers");

        self.vertices.clear();
        self.indices.clear();
        self.commands.clear();

        let commands = paint.calls().iter().map(|mesh| {
            let vertices = mesh.vertices.iter().map(|vertex| Vertex {
                pos: vertex.position,
                texcoord: vertex.texcoord,
                color: vertex.color,
            });

            let base = self.vertices.len() as u32;
            let indices = mesh.indices.iter().map(|&index| base + index as u32);

            let start = self.indices.len() as u32;
            let end = start + indices.len() as u32;

            self.vertices.extend(vertices);
            self.indices.extend(indices);

            let texture = mesh
                .texture
                .and_then(|index| self.textures.get(&index))
                .unwrap_or(&self.default_texture);

            let sampler = self.samplers.get(texture.min_filter, texture.mag_filter);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("yakui Bind Group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

            DrawCommand {
                index_range: start..end,
                bind_group,
                pipeline: mesh.pipeline,
            }
        });

        self.commands.extend(commands);
    }

    fn update_textures(&mut self, device: &wgpu::Device, paint: &PaintDom, queue: &wgpu::Queue) {
        profiling::scope!("update_textures");

        for (id, texture) in paint.textures() {
            match self.textures.get_mut(&id) {
                Some(existing) => existing.update_if_newer(device, queue, texture),
                None => {
                    self.textures
                        .insert(id, GpuTexture::new(device, queue, texture));
                }
            }
        }
    }
}

struct DrawCommand {
    index_range: Range<u32>,
    bind_group: wgpu::BindGroup,
    pipeline: Pipeline,
}
