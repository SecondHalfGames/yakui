#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]

mod bindgroup_cache;
mod buffer;
mod pipeline_cache;
mod samplers;
mod texture;

use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;

use buffer::Buffer;
use bytemuck::{Pod, Zeroable};
use glam::UVec2;
use thunderdome::{Arena, Index};
use yakui_core::geometry::{Rect, Vec2, Vec4};
use yakui_core::paint::{
    PaintCall, PaintDom, PaintLimits, Pipeline, Texture, TextureChange, TextureFormat,
    UserPaintCallId, Vertex as YakuiVertex, YakuiPaintCall,
};
use yakui_core::{ManagedTextureId, TextureId, Yakui};

use self::bindgroup_cache::TextureBindgroupCache;
use self::bindgroup_cache::TextureBindgroupCacheEntry;
use self::pipeline_cache::PipelineCache;
use self::samplers::Samplers;
use self::texture::{GpuManagedTexture, GpuTexture};

pub struct YakuiDrawCall {
    pub index_range: Range<u32>,
    pub bind_group_entry: Option<TextureBindgroupCacheEntry>,
    pub pipeline: Pipeline,
}

pub enum DrawCall {
    Yakui(YakuiDrawCall),
    User(UserPaintCallId),
}

pub struct YakuiWgpu {
    pub limits: PaintLimits,

    pub main_pipeline: PipelineCache,
    pub text_pipeline: PipelineCache,

    premul_pipeline: wgpu::RenderPipeline,
    premul_bind_group_layout: wgpu::BindGroupLayout,

    samplers: Samplers,
    textures: Arena<GpuTexture>,
    managed_textures: HashMap<ManagedTextureId, GpuManagedTexture>,
    pub texture_bindgroup_cache: TextureBindgroupCache,

    pub vertices: Buffer,
    pub indices: Buffer,
}

#[derive(Debug, Clone)]
pub struct SurfaceInfo<'a> {
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub color_attachment: wgpu::RenderPassColorAttachment<'a>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    pub position: Vec2,
    pub texcoord: Vec2,
    pub color: Vec4,
}

impl From<&YakuiVertex> for Vertex {
    fn from(y: &YakuiVertex) -> Self {
        Self {
            position: y.position,
            texcoord: y.texcoord,
            color: y.color,
        }
    }
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

impl YakuiWgpu {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let limits = PaintLimits {
            max_texture_size_1d: device.limits().max_texture_dimension_1d,
            max_texture_size_2d: device.limits().max_texture_dimension_2d,
            max_texture_size_3d: device.limits().max_texture_dimension_3d,
        };

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
            label: Some("yakui Main Pipeline Layout"),
            bind_group_layouts: &[&layout],
            immediate_size: 0,
        });

        let main_pipeline = PipelineCache::new(pipeline_layout);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("yakui Text Pipeline Layout"),
            bind_group_layouts: &[&layout],
            immediate_size: 0,
        });

        let text_pipeline = PipelineCache::new(pipeline_layout);

        let samplers = Samplers::new(device);

        let premul_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("yakui Premultiply Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let premul_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("yakui Premultiply Texture Pipeline Layout"),
                bind_group_layouts: &[&premul_bind_group_layout],
                immediate_size: 0,
            });

        let premul_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/premul.wgsl").into()),
        });

        let premul_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yakui Premultiply Texture Pipeline"),
            layout: Some(&premul_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &premul_shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &premul_shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });

        let default_texture_data =
            Texture::new(TextureFormat::Rgba8Srgb, UVec2::new(1, 1), vec![255; 4]);
        let default_texture = GpuManagedTexture::new(
            device,
            queue,
            &default_texture_data,
            &premul_pipeline,
            &premul_bind_group_layout,
            &samplers,
        );
        let default_bindgroup = bindgroup_cache::bindgroup(
            device,
            &layout,
            &samplers,
            &default_texture.view,
            default_texture.min_filter,
            default_texture.mag_filter,
            wgpu::MipmapFilterMode::Nearest,
            wgpu::AddressMode::ClampToEdge,
        );

        Self {
            limits,
            main_pipeline,
            text_pipeline,
            samplers,
            premul_pipeline,
            premul_bind_group_layout,
            textures: Arena::new(),
            managed_textures: HashMap::new(),

            texture_bindgroup_cache: TextureBindgroupCache::new(layout, default_bindgroup),
            vertices: Buffer::new(wgpu::BufferUsages::VERTEX),
            indices: Buffer::new(wgpu::BufferUsages::INDEX),
        }
    }

    pub fn set_paint_limits(&self, state: &mut Yakui) {
        state.set_paint_limit(self.limits);
    }

    /// Creates a `TextureId` from an existing wgpu texture that then be used by
    /// any yakui widgets.
    pub fn add_texture(
        &mut self,
        view: impl Into<Arc<wgpu::TextureView>>,
        min_filter: wgpu::FilterMode,
        mag_filter: wgpu::FilterMode,
        mipmap_filter: wgpu::MipmapFilterMode,
        address_mode: wgpu::AddressMode,
    ) -> TextureId {
        let index = self.textures.insert(GpuTexture {
            view: view.into(),
            min_filter,
            mag_filter,
            mipmap_filter,
            address_mode,
        });
        TextureId::User(index.to_bits())
    }

    /// Update an existing texture with a new texture view.
    ///
    /// ## Panics
    ///
    /// Will panic if `TextureId` was not created from a previous call to
    /// `add_texture`.
    pub fn update_texture(&mut self, id: TextureId, view: impl Into<Arc<wgpu::TextureView>>) {
        let index = match id {
            TextureId::User(bits) => Index::from_bits(bits).expect("invalid user texture"),
            _ => panic!("invalid user texture"),
        };

        let existing = self
            .textures
            .get_mut(index)
            .expect("user texture does not exist");
        existing.view = view.into();
    }

    #[must_use = "YakuiWgpu::paint returns a command buffer which MUST be submitted to wgpu."]
    pub fn paint(
        &mut self,
        state: &mut Yakui,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: SurfaceInfo<'_>,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("yakui Encoder"),
        });

        self.paint_with_encoder(state, device, queue, &mut encoder, surface);

        encoder.finish()
    }

    pub fn paint_with_encoder(
        &mut self,
        state: &mut Yakui,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface: SurfaceInfo<'_>,
    ) {
        profiling::scope!("yakui-wgpu paint_with_encoder");

        crate::paint(self, state, device, queue, encoder, surface)
    }

    pub fn build_draw_call(
        &mut self,
        device: &wgpu::Device,
        clip: Rect,
        call: &YakuiPaintCall,
    ) -> (Rect, DrawCall) {
        let vertices = call.vertices.iter().map(Vertex::from);

        let base = self.vertices.len() as u32;
        let indices = call.indices.iter().map(|&index| base + index as u32);

        let start = self.indices.len() as u32;
        let end = start + indices.len() as u32;

        self.vertices.extend(vertices);
        self.indices.extend(indices);

        let bind_group_entry = call
            .texture
            .and_then(|id| match id {
                TextureId::Managed(managed) => {
                    let texture = self.managed_textures.get(&managed)?;
                    Some((
                        id,
                        &texture.view,
                        texture.min_filter,
                        texture.mag_filter,
                        wgpu::MipmapFilterMode::Nearest,
                        texture.address_mode,
                    ))
                }
                TextureId::User(bits) => {
                    let index = Index::from_bits(bits)?;
                    let texture = self.textures.get(index)?;
                    Some((
                        id,
                        &texture.view,
                        texture.min_filter,
                        texture.mag_filter,
                        texture.mipmap_filter,
                        texture.address_mode,
                    ))
                }
            })
            .map(
                |(id, view, min_filter, mag_filter, mipmap_filter, address_mode)| {
                    let entry = TextureBindgroupCacheEntry {
                        id,
                        min_filter,
                        mag_filter,
                        mipmap_filter,
                        address_mode,
                    };
                    self.texture_bindgroup_cache
                        .update(device, entry, view, &self.samplers);

                    entry
                },
            );

        (
            clip,
            DrawCall::Yakui(YakuiDrawCall {
                index_range: start..end,
                bind_group_entry,
                pipeline: call.pipeline,
            }),
        )
    }

    pub fn draw_yakui(
        texture_bindgroup_cache: &TextureBindgroupCache,
        render_pass: &mut wgpu::RenderPass,
        main_pipeline: &wgpu::RenderPipeline,
        text_pipeline: &wgpu::RenderPipeline,
        call: YakuiDrawCall,
    ) {
        profiling::scope!("yakui-wgpu draw_yakui");

        match call.pipeline {
            Pipeline::Main => render_pass.set_pipeline(main_pipeline),
            Pipeline::Text => render_pass.set_pipeline(text_pipeline),
        }

        let bindgroup = call
            .bind_group_entry
            .map(|entry| texture_bindgroup_cache.get(&entry))
            .unwrap_or(&texture_bindgroup_cache.default);

        render_pass.set_bind_group(0, bindgroup, &[]);
        render_pass.draw_indexed(call.index_range.clone(), 0, 0..1);
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        paint: &PaintDom,
        queue: &wgpu::Queue,
    ) {
        profiling::scope!("update_textures");

        for (id, texture) in paint.textures() {
            self.managed_textures.entry(id).or_insert_with(|| {
                GpuManagedTexture::new(
                    device,
                    queue,
                    texture,
                    &self.premul_pipeline,
                    &self.premul_bind_group_layout,
                    &self.samplers,
                )
            });
        }

        for (id, change) in paint.texture_edits() {
            match change {
                TextureChange::Added => {
                    let texture = paint.texture(id).unwrap();
                    self.managed_textures.insert(
                        id,
                        GpuManagedTexture::new(
                            device,
                            queue,
                            texture,
                            &self.premul_pipeline,
                            &self.premul_bind_group_layout,
                            &self.samplers,
                        ),
                    );
                }

                TextureChange::Removed => {
                    self.managed_textures.remove(&id);
                }

                TextureChange::Modified => {
                    if let Some(existing) = self.managed_textures.get_mut(&id) {
                        let texture = paint.texture(id).unwrap();
                        existing.update(
                            device,
                            queue,
                            texture,
                            &self.premul_pipeline,
                            &self.premul_bind_group_layout,
                            &self.samplers,
                        );
                    }
                }
            }
        }
    }
}

fn make_main_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    samples: u32,
) -> wgpu::RenderPipeline {
    let main_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Main Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("yakui Main Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &main_shader,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[Vertex::DESCRIPTOR],
        },
        fragment: Some(wgpu::FragmentState {
            module: &main_shader,
            entry_point: None,
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}

fn make_text_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    samples: u32,
) -> wgpu::RenderPipeline {
    let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Text Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/text.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("yakui Text Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &text_shader,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[Vertex::DESCRIPTOR],
        },
        fragment: Some(wgpu::FragmentState {
            module: &text_shader,
            entry_point: None,
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}

pub fn main_pipeline<'a>(
    main_pipeline: &'a mut PipelineCache,
    device: &wgpu::Device,
    surface: &SurfaceInfo<'_>,
) -> &'a wgpu::RenderPipeline {
    main_pipeline.get(
        device,
        surface.format,
        surface.sample_count,
        make_main_pipeline,
    )
}

pub fn text_pipeline<'a>(
    text_pipeline: &'a mut PipelineCache,
    device: &wgpu::Device,
    surface: &SurfaceInfo<'_>,
) -> &'a wgpu::RenderPipeline {
    text_pipeline.get(
        device,
        surface.format,
        surface.sample_count,
        make_text_pipeline,
    )
}

pub fn paint(
    yakui_wgpu: &mut YakuiWgpu,
    state: &mut Yakui,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface: SurfaceInfo<'_>,
) {
    // --- yakui ---
    yakui_wgpu.set_paint_limits(state);
    let paint = state.paint();

    yakui_wgpu.update_textures(device, paint, queue);

    // If there's nothing to paint, well... don't paint!
    let layers = &paint.layers;
    if layers.iter().all(|layer| layer.calls.is_empty()) {
        return;
    }

    // If the surface has a size of zero, well... don't paint either!
    if paint.surface_size().x == 0.0 || paint.surface_size().y == 0.0 {
        return;
    }

    yakui_wgpu.vertices.clear();
    yakui_wgpu.indices.clear();
    yakui_wgpu.texture_bindgroup_cache.clear();

    let mut draw_calls = Vec::with_capacity(layers.len());
    // --- yakui ---

    for (clip, call) in layers.iter().flat_map(|layer| &layer.calls) {
        match call {
            PaintCall::Internal(call) => {
                draw_calls.push(yakui_wgpu.build_draw_call(device, *clip, call));
            }
            PaintCall::User(_) => {
                panic!("yakui does not handle User PaintCall's by default. Please set up your own rendering logic instead.");
            }
        }
    }

    // --- yakui ---
    let vertices = yakui_wgpu.vertices.upload(device, queue);
    let indices = yakui_wgpu.indices.upload(device, queue);
    // --- yakui ---

    {
        // --- yakui ---
        let main_pipeline = main_pipeline(&mut yakui_wgpu.main_pipeline, device, &surface);
        let text_pipeline = text_pipeline(&mut yakui_wgpu.text_pipeline, device, &surface);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("yakui Render Pass"),
            color_attachments: &[Some(surface.color_attachment)],
            ..Default::default()
        });

        render_pass.set_vertex_buffer(0, vertices.slice(..));
        render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

        let surface = paint.surface_size().as_uvec2();
        render_pass.set_viewport(0.0, 0.0, surface.x as f32, surface.y as f32, 0.0, 1.0);

        let mut last_clip = None;
        // --- yakui ---

        for (clip, draw_call) in draw_calls {
            // --- yakui ---
            if Some(clip) != last_clip {
                last_clip = Some(clip);

                let surface = paint.surface_size().as_uvec2();

                let pos = clip.pos().as_uvec2();
                let size = clip.size().as_uvec2();

                let max = (pos + size).min(surface);
                let size = UVec2::new(max.x.saturating_sub(pos.x), max.y.saturating_sub(pos.y));

                // If the scissor rect isn't valid, we can skip this
                // entire draw call.
                if pos.x > surface.x || pos.y > surface.y || size.x == 0 || size.y == 0 {
                    continue;
                }

                render_pass.set_scissor_rect(pos.x, pos.y, size.x, size.y);
            }
            // --- yakui ---

            match draw_call {
                DrawCall::Yakui(call) => {
                    YakuiWgpu::draw_yakui(
                        &yakui_wgpu.texture_bindgroup_cache,
                        &mut render_pass,
                        main_pipeline,
                        text_pipeline,
                        call,
                    );
                }
                DrawCall::User(_) => {
                    panic!("yakui does not handle User PaintCall's by default. Please set up your own rendering logic instead.");
                }
            }
        }
    }
}
