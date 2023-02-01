mod buffer;
mod descriptors;
mod util;
mod vulkan_context;
mod vulkan_texture;

use buffer::Buffer;
use bytemuck::{bytes_of, Pod, Zeroable};
use descriptors::Descriptors;
use glam::UVec2;
use std::{collections::HashMap, ffi::CStr, io::Cursor};
use vulkan_context::VulkanContext;
use vulkan_texture::{VulkanTexture, VulkanTextureCreateInfo, NO_TEXTURE_ID};
use yakui::{paint::Vertex as YakuiVertex, ManagedTextureId};

use ash::{util::read_spv, vk};

pub struct YakuiVulkan {
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    render_surface: RenderSurface,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    index_buffer: Buffer<u32>,
    vertex_buffer: Buffer<Vertex>,
    initial_textures_synced: bool,
    managed_textures: HashMap<ManagedTextureId, VulkanTexture>,
    user_textures: thunderdome::Arena<VulkanTexture>,
    descriptors: Descriptors,
}

#[derive(Clone)]
pub struct RenderSurface {
    pub resolution: vk::Extent2D,
    pub format: vk::Format,
    pub image_views: Vec<vk::ImageView>,
}

#[derive(Clone, Copy, Debug)]
pub struct DrawCall {
    index_offset: u32,
    index_count: u32,
    clip: Option<yakui::geometry::Rect>,
    texture_id: u32,
    workflow: Workflow,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PushConstant {
    texture_id: u32,
    workflow: Workflow,
}

unsafe impl Zeroable for PushConstant {}
unsafe impl Pod for PushConstant {}

impl PushConstant {
    pub fn new(texture_id: u32, workflow: Workflow) -> Self {
        Self {
            texture_id,
            workflow,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Workflow {
    Main,
    Text,
}

unsafe impl bytemuck::Zeroable for Workflow {}
unsafe impl bytemuck::Pod for Workflow {}

impl From<yakui::paint::Pipeline> for Workflow {
    fn from(p: yakui::paint::Pipeline) -> Self {
        match p {
            yakui::paint::Pipeline::Main => Workflow::Main,
            yakui::paint::Pipeline::Text => Workflow::Text,
            _ => panic!("Unknown pipeline {p:?}"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    position: glam::Vec2,
    texcoord: glam::Vec2,
    color: glam::Vec4,
}

impl Vertex {
    pub fn new(position: glam::Vec2, texcoord: glam::Vec2, color: glam::Vec4) -> Self {
        Self {
            position,
            texcoord,
            color,
        }
    }
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

impl YakuiVulkan {
    pub fn new(vulkan_context: &VulkanContext, render_surface: RenderSurface) -> Self {
        let device = vulkan_context.device;
        let descriptors = Descriptors::new(vulkan_context);

        // TODO: Don't write directly to the present surface..
        let renderpass_attachments = [vk::AttachmentDescription {
            format: render_surface.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let framebuffers: Vec<vk::Framebuffer> = render_surface
            .image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(render_surface.resolution.width)
                    .height(render_surface.resolution.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        let index_buffer = Buffer::new(vulkan_context, vk::BufferUsageFlags::INDEX_BUFFER, &[]);
        let vertex_buffer = Buffer::new(vulkan_context, vk::BufferUsageFlags::VERTEX_BUFFER, &[]);

        let mut vertex_spv_file = Cursor::new(&include_bytes!("../shaders/main.vert.spv")[..]);
        let mut frag_spv_file = Cursor::new(&include_bytes!("../shaders/main.frag.spv")[..]);

        let vertex_code =
            read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
        let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

        let frag_code =
            read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

        let vertex_shader_module = unsafe {
            device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Vertex shader module error")
        };

        let fragment_shader_module = unsafe {
            device
                .create_shader_module(&frag_shader_info, None)
                .expect("Fragment shader module error")
        };

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::builder()
                        .push_constant_ranges(std::slice::from_ref(
                            &vk::PushConstantRange::builder()
                                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                                .size(std::mem::size_of::<PushConstant>() as _),
                        ))
                        .set_layouts(std::slice::from_ref(&descriptors.layout)),
                    None,
                )
                .unwrap()
        };

        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

        let vertex_input_attribute_descriptions = [
            // position
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            // UV / texcoords
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 8,
            },
            // color
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 16,
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: render_surface.resolution.width as f32,
            height: render_surface.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [render_surface.resolution.into()];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass);

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        let graphics_pipeline = graphics_pipelines[0];

        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Self {
            render_pass,
            descriptors,
            framebuffers,
            render_surface,
            pipeline_layout,
            graphics_pipeline,
            index_buffer,
            vertex_buffer,
            user_textures: Default::default(),
            managed_textures: Default::default(),
            initial_textures_synced: false,
        }
    }

    pub fn paint(
        &mut self,
        state: &mut yakui_core::Yakui,
        vulkan_context: &VulkanContext,
        present_index: u32,
    ) {
        let paint = state.paint();

        self.update_textures(vulkan_context, paint);

        // If there's nothing to paint, well.. don't paint!
        if paint.calls().is_empty() {
            return;
        }

        let draw_calls = self.build_draw_calls(vulkan_context, paint);

        self.render(vulkan_context, present_index, &draw_calls);
    }

    fn render(&self, vulkan_context: &VulkanContext, present_index: u32, draw_calls: &[DrawCall]) {
        let device = vulkan_context.device;
        let command_buffer = vulkan_context.draw_command_buffer;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let surface = &self.render_surface;

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[present_index as usize])
            .render_area(surface.resolution.into())
            .clear_values(&clear_values);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: surface.resolution.width as f32,
            height: surface.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let surface_size = UVec2::new(surface.resolution.width, surface.resolution.height);

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            let default_scissor = [surface.resolution.into()];
            device.cmd_set_scissor(command_buffer, 0, &default_scissor);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.handle,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptors.set),
                &[],
            );
            let mut last_clip = None;
            for draw_call in draw_calls {
                if draw_call.clip != last_clip {
                    last_clip = draw_call.clip;

                    // TODO - do this when processing draw calls
                    match draw_call.clip {
                        Some(rect) => {
                            let pos = rect.pos().as_uvec2();
                            let size = rect.size().as_uvec2();

                            let max = (pos + size).min(surface_size);
                            let size = UVec2::new(
                                max.x.saturating_sub(pos.x),
                                max.y.saturating_sub(pos.y),
                            );

                            // If the scissor rect isn't valid, we can skip this
                            // entire draw call.
                            if pos.x > surface_size.x
                                || pos.y > surface_size.y
                                || size.x == 0
                                || size.y == 0
                            {
                                continue;
                            }

                            let scissors = [vk::Rect2D {
                                offset: vk::Offset2D {
                                    x: pos.x as _,
                                    y: pos.y as _,
                                },
                                extent: surface.resolution,
                            }];
                            device.cmd_set_scissor(command_buffer, 0, &scissors);
                        }
                        None => {
                            device.cmd_set_scissor(command_buffer, 0, &default_scissor);
                        }
                    }
                }
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytes_of(&PushConstant::new(draw_call.texture_id, draw_call.workflow)),
                );
                device.cmd_draw_indexed(
                    command_buffer,
                    draw_call.index_count,
                    1,
                    draw_call.index_offset,
                    0,
                    1,
                );
            }
            device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn add_user_texture(
        &mut self,
        vulkan_context: &VulkanContext,
        texture_create_info: VulkanTextureCreateInfo<Vec<u8>>,
    ) -> yakui::TextureId {
        let texture =
            VulkanTexture::new(vulkan_context, &mut self.descriptors, texture_create_info);
        yakui::TextureId::User(self.user_textures.insert(texture).to_bits())
    }

    pub fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            device.device_wait_idle().unwrap();
            self.descriptors.cleanup(device);
            for (_, texture) in self.managed_textures.drain() {
                texture.cleanup(device);
            }
            for (_, texture) in self.user_textures.drain() {
                texture.cleanup(device);
            }
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            self.index_buffer.cleanup(device);
            self.vertex_buffer.cleanup(device);
            for framebuffer in &self.framebuffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            device.destroy_render_pass(self.render_pass, None);
        }
    }

    fn update_textures(&mut self, vulkan_context: &VulkanContext, paint: &yakui::paint::PaintDom) {
        use yakui::paint::TextureChange;
        if !self.initial_textures_synced {
            self.initial_textures_synced = true;
            for (id, texture) in paint.textures() {
                let texture = VulkanTexture::from_yakui_texture(
                    vulkan_context,
                    &mut self.descriptors,
                    texture,
                );
                self.managed_textures.insert(id, texture);
            }

            return;
        }

        for (id, change) in paint.texture_edits() {
            match change {
                TextureChange::Added => {
                    let texture = paint.texture(id).unwrap();
                    let texture = VulkanTexture::from_yakui_texture(
                        vulkan_context,
                        &mut self.descriptors,
                        texture,
                    );
                    self.managed_textures.insert(id, texture);
                }

                TextureChange::Removed => {
                    if let Some(removed) = self.managed_textures.remove(&id) {
                        unsafe { removed.cleanup(vulkan_context.device) };
                    }
                }

                TextureChange::Modified => {
                    if let Some(old) = self.managed_textures.remove(&id) {
                        unsafe { old.cleanup(vulkan_context.device) };
                    }
                    let new = paint.texture(id).unwrap();
                    let texture = VulkanTexture::from_yakui_texture(
                        vulkan_context,
                        &mut self.descriptors,
                        new,
                    );
                    self.managed_textures.insert(id, texture);
                }
            }
        }
    }

    fn build_draw_calls(
        &mut self,
        vulkan_context: &VulkanContext,
        paint: &yakui::paint::PaintDom,
    ) -> Vec<DrawCall> {
        let mut vertices: Vec<Vertex> = Default::default();
        let mut indices: Vec<u32> = Default::default();
        let mut draw_calls: Vec<DrawCall> = Default::default();

        for mesh in paint.calls() {
            let base = vertices.len() as u32;
            let index_offset = indices.len() as u32;
            let index_count = mesh.indices.len() as u32;

            for index in &mesh.indices {
                indices.push(*index as u32 + base);
            }
            for vertex in &mesh.vertices {
                vertices.push(vertex.into())
            }

            let texture_id = mesh
                .texture
                .and_then(|id| match id {
                    yakui::TextureId::Managed(managed) => {
                        let texture = self.managed_textures.get(&managed)?;
                        Some(texture.id)
                    }
                    yakui::TextureId::User(bits) => {
                        let texture = self
                            .user_textures
                            .get(thunderdome::Index::from_bits(bits)?)?;
                        Some(texture.id)
                    }
                })
                .unwrap_or(NO_TEXTURE_ID);

            draw_calls.push(DrawCall {
                index_offset,
                index_count,
                clip: mesh.clip,
                texture_id,
                workflow: mesh.pipeline.into(),
            });
        }

        unsafe {
            self.index_buffer.overwrite(vulkan_context, &indices);
            self.vertex_buffer.overwrite(vulkan_context, &vertices);
        }

        draw_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash::{
        extensions::khr::{Surface, Swapchain},
        vk,
    };
    use glam::Vec2;
    use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
    use std::{cell::RefCell, ffi::CStr};
    use winit::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::ControlFlow,
        platform::run_return::EventLoopExtRunReturn,
        window::WindowBuilder,
    };
    use yakui::image;

    const MONKEY_PNG: &[u8] = include_bytes!("../../demo/assets/monkey.png");
    const DOG_PNG: &[u8] = include_bytes!("../assets/dog.png");

    #[derive(Debug, Clone)]
    struct GuiState {
        monkey: yakui::ManagedTextureId,
        dog: yakui::TextureId,
        which_image: WhichImage,
    }

    #[derive(Debug, Clone)]
    enum WhichImage {
        Monkey,
        Dog,
    }

    #[test]
    /// Simple smoke test to make sure render screen properly pixel Vulkan.
    ///
    /// Scoped to only run on Windows for simplicity.
    #[cfg(target_os = "windows")]
    fn it_works() {
        let (width, height) = (500, 500);
        let vulkan_test = VulkanTest::new(width, height);
        let render_surface = RenderSurface {
            resolution: vk::Extent2D { width, height },
            format: vulkan_test.surface_format.format,
            image_views: vulkan_test.present_image_views.clone(),
        };
        let mut yak = yakui::Yakui::new();
        yak.set_surface_size([width as f32, height as f32].into());
        let vulkan_context = VulkanContext::new(
            &vulkan_test.device,
            vulkan_test.present_queue,
            vulkan_test.draw_command_buffer,
            vulkan_test.command_pool,
            vulkan_test.device_memory_properties,
        );
        let mut yakui_vulkan = YakuiVulkan::new(&vulkan_context, render_surface);
        let gui_state = GuiState {
            monkey: yak.add_texture(create_yakui_texture(
                MONKEY_PNG,
                yakui::paint::TextureFilter::Linear,
            )),
            dog: yakui_vulkan.add_user_texture(
                &vulkan_context,
                create_vulkan_texture_info(DOG_PNG, vk::Filter::LINEAR),
            ),
            which_image: WhichImage::Monkey,
        };

        vulkan_test.render_loop(yak, gui_state, |buffer_index, yak, gui_state| {
            yak.start();
            gui(&gui_state);
            yak.finish();

            yakui_vulkan.paint(yak, &vulkan_context, buffer_index);
        });

        yakui_vulkan.cleanup(&vulkan_test.device);
    }

    fn create_vulkan_texture_info(
        compressed_image_bytes: &[u8],
        filter: vk::Filter,
    ) -> VulkanTextureCreateInfo<Vec<u8>> {
        let image = image::load_from_memory(compressed_image_bytes)
            .unwrap()
            .into_rgba8();
        let resolution = vk::Extent2D {
            width: image.width(),
            height: image.height(),
        };

        VulkanTextureCreateInfo::new(
            image.into_raw(),
            vk::Format::R8G8B8A8_UNORM,
            resolution,
            filter,
            filter,
        )
    }

    fn create_yakui_texture(
        compressed_image_bytes: &[u8],
        filter: yakui::paint::TextureFilter,
    ) -> yakui::paint::Texture {
        let image = image::load_from_memory(compressed_image_bytes)
            .unwrap()
            .into_rgba8();
        let size = UVec2::new(image.width(), image.height());

        let mut texture = yakui::paint::Texture::new(
            yakui::paint::TextureFormat::Rgba8Srgb,
            size,
            image.into_raw(),
        );
        texture.mag_filter = filter;
        texture
    }

    fn gui(gui_state: &GuiState) {
        use yakui::{column, label, row, text, widgets::Text, Color};
        let (animal, texture): (&'static str, yakui::TextureId) = match gui_state.which_image {
            WhichImage::Monkey => ("monkye", gui_state.monkey.into()),
            WhichImage::Dog => ("dog haha good boy", gui_state.dog.into()),
        };
        column(|| {
            row(|| {
                label("Hello, world!");

                let mut text = Text::new(48.0, "colored text!");
                text.style.color = Color::RED;
                text.show();
            });

            text(96.0, format!("look it is a {animal}"));

            image(texture, Vec2::new(400.0, 400.0));
        });
    }

    struct VulkanTest {
        pub _window: winit::window::Window,
        pub _entry: ash::Entry,
        pub device: ash::Device,
        pub instance: ash::Instance,
        pub surface_loader: Surface,
        pub swapchain_loader: Swapchain,
        pub event_loop: RefCell<winit::event_loop::EventLoop<()>>,
        pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,

        pub present_queue: vk::Queue,

        pub surface: vk::SurfaceKHR,
        pub surface_format: vk::SurfaceFormatKHR,

        pub swapchain: vk::SwapchainKHR,
        pub present_image_views: Vec<vk::ImageView>,

        pub command_pool: vk::CommandPool,
        pub draw_command_buffer: vk::CommandBuffer,

        pub present_complete_semaphore: vk::Semaphore,
        pub rendering_complete_semaphore: vk::Semaphore,

        pub draw_commands_reuse_fence: vk::Fence,
        pub setup_commands_reuse_fence: vk::Fence,
    }

    impl VulkanTest {
        pub fn render_loop<F: FnMut(u32, &mut yakui::Yakui, &GuiState)>(
            &self,
            mut yak: yakui::Yakui,
            mut gui_state: GuiState,
            mut f: F,
        ) {
            self.event_loop
                .borrow_mut()
                .run_return(|event, _, control_flow| {
                    *control_flow = ControlFlow::Poll;
                    match event {
                        Event::WindowEvent {
                            event:
                                WindowEvent::CloseRequested
                                | WindowEvent::KeyboardInput {
                                    input:
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::Escape),
                                            ..
                                        },
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        Event::MainEventsCleared => {
                            let framebuffer_index = self.render_begin();
                            f(framebuffer_index, &mut yak, &gui_state);
                            self.render_end(framebuffer_index);
                        }
                        Event::WindowEvent {
                            event:
                                WindowEvent::KeyboardInput {
                                    input:
                                        KeyboardInput {
                                            state: ElementState::Released,
                                            virtual_keycode,
                                            ..
                                        },
                                    ..
                                },
                            ..
                        } => match virtual_keycode {
                            Some(VirtualKeyCode::A) => {
                                gui_state.which_image = match &gui_state.which_image {
                                    WhichImage::Monkey => WhichImage::Dog,
                                    WhichImage::Dog => WhichImage::Monkey,
                                }
                            }
                            _ => {}
                        },
                        _ => (),
                    }
                });
        }

        /// Bring up a `winit` window and all the Vulkan pomp and ceremony required to render things.
        /// Vulkan Broadly lifted from: https://github.com/ash-rs/ash/blob/0.37.2/examples/src/lib.rs
        fn new(window_width: u32, window_height: u32) -> Self {
            use winit::{
                event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
            };

            let event_loop = EventLoopBuilder::new().with_any_thread(true).build(); // necessary because tests are in a separate thread
            let window = WindowBuilder::new()
                .with_title("Yakui Vulkan - Test")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    f64::from(window_width),
                    f64::from(window_height),
                ))
                .build(&event_loop)
                .unwrap();
            let entry = ash::Entry::linked();
            let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };

            let appinfo = vk::ApplicationInfo::builder()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 3, 0));

            let extension_names =
                ash_window::enumerate_required_extensions(window.raw_display_handle())
                    .unwrap()
                    .to_vec();

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_extension_names(&extension_names);

            let instance = unsafe {
                entry
                    .create_instance(&create_info, None)
                    .expect("Instance creation error")
            };

            let surface = unsafe {
                ash_window::create_surface(
                    &entry,
                    &instance,
                    window.raw_display_handle(),
                    window.raw_window_handle(),
                    None,
                )
                .unwrap()
            };

            let pdevices = unsafe {
                instance
                    .enumerate_physical_devices()
                    .expect("Physical device error")
            };
            let surface_loader = Surface::new(&entry, &instance);
            let (pdevice, queue_family_index) = unsafe {
                pdevices
                    .iter()
                    .find_map(|pdevice| {
                        instance
                            .get_physical_device_queue_family_properties(*pdevice)
                            .iter()
                            .enumerate()
                            .find_map(|(index, info)| {
                                let supports_graphic_and_surface =
                                    info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                        && surface_loader
                                            .get_physical_device_surface_support(
                                                *pdevice,
                                                index as u32,
                                                surface,
                                            )
                                            .unwrap();
                                if supports_graphic_and_surface {
                                    Some((*pdevice, index))
                                } else {
                                    None
                                }
                            })
                    })
                    .expect("Couldn't find suitable device.")
            };
            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let mut descriptor_indexing_features =
                vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
                    .shader_sampled_image_array_non_uniform_indexing(true)
                    .descriptor_binding_partially_bound(true)
                    .descriptor_binding_variable_descriptor_count(true);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .push_next(&mut descriptor_indexing_features);

            let device = unsafe {
                instance
                    .create_device(pdevice, &device_create_info, None)
                    .unwrap()
            };

            let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

            let surface_format = unsafe {
                surface_loader
                    .get_physical_device_surface_formats(pdevice, surface)
                    .unwrap()[0]
            };

            let surface_capabilities = unsafe {
                surface_loader
                    .get_physical_device_surface_capabilities(pdevice, surface)
                    .unwrap()
            };
            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }
            let surface_resolution = match surface_capabilities.current_extent.width {
                std::u32::MAX => vk::Extent2D {
                    width: window_width,
                    height: window_height,
                },
                _ => surface_capabilities.current_extent,
            };
            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };
            let present_modes = unsafe {
                surface_loader
                    .get_physical_device_surface_present_modes(pdevice, surface)
                    .unwrap()
            };
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            let swapchain_loader = Swapchain::new(&instance, &device);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = unsafe {
                swapchain_loader
                    .create_swapchain(&swapchain_create_info, None)
                    .unwrap()
            };

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = unsafe {
                device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .unwrap()
            };
            let draw_command_buffer = command_buffers[0];

            let present_images =
                unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
            let present_image_views: Vec<vk::ImageView> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    unsafe { device.create_image_view(&create_view_info, None).unwrap() }
                })
                .collect();

            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_commands_reuse_fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Create fence failed.")
            };
            let setup_commands_reuse_fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Create fence failed.")
            };

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap()
            };
            let rendering_complete_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap()
            };

            let device_memory_properties =
                unsafe { instance.get_physical_device_memory_properties(pdevice) };

            Self {
                _window: window,
                device,
                present_queue,
                _entry: entry,
                instance,
                surface_loader,
                swapchain_loader,
                device_memory_properties,
                event_loop: RefCell::new(event_loop),
                surface,
                surface_format,
                swapchain,
                present_image_views,
                command_pool: pool,
                draw_command_buffer,
                present_complete_semaphore,
                rendering_complete_semaphore,
                draw_commands_reuse_fence,
                setup_commands_reuse_fence,
            }
        }

        fn render_begin(&self) -> u32 {
            let (present_index, _) = unsafe {
                self.swapchain_loader
                    .acquire_next_image(
                        self.swapchain,
                        std::u64::MAX,
                        self.present_complete_semaphore,
                        vk::Fence::null(),
                    )
                    .unwrap()
            };

            let device = &self.device;
            unsafe {
                device
                    .wait_for_fences(
                        std::slice::from_ref(&self.draw_commands_reuse_fence),
                        true,
                        std::u64::MAX,
                    )
                    .unwrap();
                device
                    .reset_fences(std::slice::from_ref(&self.draw_commands_reuse_fence))
                    .unwrap();
                device
                    .reset_command_buffer(
                        self.draw_command_buffer,
                        vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                    )
                    .unwrap();
                device
                    .begin_command_buffer(
                        self.draw_command_buffer,
                        &vk::CommandBufferBeginInfo::builder()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                    )
                    .unwrap();
            }
            present_index
        }

        fn render_end(&self, present_index: u32) {
            let device = &self.device;
            unsafe {
                device.end_command_buffer(self.draw_command_buffer).unwrap();
                let swapchains = [self.swapchain];
                let image_indices = [present_index];
                let submit_info = vk::SubmitInfo::builder()
                    .wait_semaphores(std::slice::from_ref(&self.present_complete_semaphore))
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .command_buffers(std::slice::from_ref(&self.draw_command_buffer))
                    .signal_semaphores(std::slice::from_ref(&self.rendering_complete_semaphore));

                device
                    .queue_submit(
                        self.present_queue,
                        std::slice::from_ref(&submit_info),
                        self.draw_commands_reuse_fence,
                    )
                    .unwrap();

                self.swapchain_loader
                    .queue_present(
                        self.present_queue,
                        &vk::PresentInfoKHR::builder()
                            .image_indices(&image_indices)
                            .wait_semaphores(std::slice::from_ref(
                                &self.rendering_complete_semaphore,
                            ))
                            .swapchains(&swapchains),
                    )
                    .unwrap()
            };
        }
    }

    impl Drop for VulkanTest {
        fn drop(&mut self) {
            unsafe {
                self.device.device_wait_idle().unwrap();
                self.device
                    .destroy_semaphore(self.present_complete_semaphore, None);
                self.device
                    .destroy_semaphore(self.rendering_complete_semaphore, None);
                self.device
                    .destroy_fence(self.draw_commands_reuse_fence, None);
                self.device
                    .destroy_fence(self.setup_commands_reuse_fence, None);
                for &image_view in self.present_image_views.iter() {
                    self.device.destroy_image_view(image_view, None);
                }
                self.device.destroy_command_pool(self.command_pool, None);
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);
                self.device.destroy_device(None);
                self.surface_loader.destroy_surface(self.surface, None);
                self.instance.destroy_instance(None);
            }
        }
    }
}
