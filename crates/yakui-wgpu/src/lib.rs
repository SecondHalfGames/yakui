#![allow(clippy::new_without_default)]

mod buffer;

use std::mem::size_of;

use buffer::Buffer;
use bytemuck::{Pod, Zeroable};
use yakui_core::{Vec2, Vec4};

pub struct State {
    pipeline: wgpu::RenderPipeline,
    vertices: Buffer,
    indices: Buffer,
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
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("yukui Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yukui Render Pipeline"),
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

        Self {
            pipeline,
            vertices: Buffer::new(wgpu::BufferUsages::VERTEX),
            indices: Buffer::new(wgpu::BufferUsages::INDEX),
        }
    }

    pub fn paint(
        &mut self,
        state: &yakui_core::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_attachment: &wgpu::TextureView,
    ) -> wgpu::CommandBuffer {
        self.vertices.clear();
        self.indices.clear();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("yakui Encoder"),
        });

        let output = state.paint();

        if output.meshes.is_empty() {
            return encoder.finish();
        }

        let draw_calls: Vec<_> = output
            .meshes
            .into_iter()
            .map(|mesh| {
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

                start..end
            })
            .collect();

        let vertices = self.vertices.upload(device, queue);
        let num_indices = self.indices.len() as u32;
        let indices = self.indices.upload(device, queue);

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

            for range in draw_calls {
                render_pass.draw_indexed(range, 0, 0..1);
            }
        }

        encoder.finish()
    }
}
