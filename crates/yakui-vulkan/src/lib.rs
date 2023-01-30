use std::{ffi::CStr, io::Cursor, marker::PhantomData, mem::align_of};

use ash::{
    util::{read_spv, Align},
    vk,
};

pub struct YakuiVulkan {
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    render_surface: RenderSurface,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    index_buffer: Buffer<u32>,
    vertex_buffer: Buffer<Vertex>,
}

#[derive(Clone)]
pub struct RenderSurface {
    pub resolution: vk::Extent2D,
    pub format: vk::Format,
    pub image_views: Vec<vk::ImageView>,
}

#[derive(Clone, Debug, Copy, Default)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

pub struct Buffer<T> {
    pub handle: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub usage: vk::BufferUsageFlags,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(
        device: &ash::Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        usage: vk::BufferUsageFlags,
        initial_data: &[T],
    ) -> Self {
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(initial_data) as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&index_buffer_info, None).unwrap() };
        let memory_req = unsafe { device.get_buffer_memory_requirements(handle) };
        let memory_index = find_memorytype_index(
            &memory_req,
            device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&allocate_info, None).unwrap() };
        let ptr = unsafe {
            device
                .map_memory(memory, 0, memory_req.size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        let mut slice = unsafe { Align::new(ptr, align_of::<T>() as u64, memory_req.size) };
        slice.copy_from_slice(initial_data);
        unsafe {
            device.unmap_memory(memory);
            device.bind_buffer_memory(handle, memory, 0).unwrap();
        }

        Buffer {
            handle,
            memory,
            usage,
            _phantom: PhantomData,
        }
    }
}

impl YakuiVulkan {
    pub fn new(
        device: &ash::Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        render_surface: RenderSurface,
    ) -> Self {
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

        let index_buffer_data = [0u32, 1, 2];
        let index_buffer = Buffer::new(
            device,
            device_memory_properties,
            vk::BufferUsageFlags::INDEX_BUFFER,
            &index_buffer_data,
        );

        let vertices = [
            Vertex {
                pos: [-1.0, 1.0, 0.0, 1.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, 0.0, 1.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                pos: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let vertex_buffer = Buffer::new(
            device,
            device_memory_properties,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &vertices,
        );

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

        let layout_create_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&layout_create_info, None)
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
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: bytemuck::offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: bytemuck::offset_of!(Vertex, color) as u32,
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
            framebuffers,
            render_surface,
            pipeline_layout,
            graphics_pipeline,
            index_buffer,
            vertex_buffer,
        }
    }

    pub fn draw(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        present_index: u32,
    ) {
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
        let scissors = [surface.resolution.into()];

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
            device.cmd_set_scissor(command_buffer, 0, &scissors);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.handle,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_draw_indexed(command_buffer, 3, 1, 0, 0, 1); // TODO: make dynamical
            device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn cleanup(&self, device: &ash::Device) {
        unsafe {
            device.device_wait_idle().unwrap();
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.free_memory(self.vertex_buffer.memory, None);
            device.destroy_buffer(self.vertex_buffer.handle, None);
            device.free_memory(self.index_buffer.memory, None);
            device.destroy_buffer(self.index_buffer.handle, None);
            for framebuffer in &self.framebuffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash::{
        extensions::khr::{Surface, Swapchain},
        vk,
    };
    use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
    use std::{cell::RefCell, ffi::CStr};
    use winit::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::ControlFlow,
        platform::run_return::EventLoopExtRunReturn,
        window::WindowBuilder,
    };

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
        let mut yakui_vulkan = YakuiVulkan::new(
            &vulkan_test.device,
            &vulkan_test.device_memory_properties,
            render_surface,
        );

        vulkan_test.render_loop(|device, command_buffer, buffer_index| {
            yakui_vulkan.draw(device, command_buffer, buffer_index)
        });

        yakui_vulkan.cleanup(&vulkan_test.device);
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

        pub pool: vk::CommandPool,
        pub draw_command_buffer: vk::CommandBuffer,

        pub present_complete_semaphore: vk::Semaphore,
        pub rendering_complete_semaphore: vk::Semaphore,

        pub draw_commands_reuse_fence: vk::Fence,
        pub setup_commands_reuse_fence: vk::Fence,
    }

    impl VulkanTest {
        pub fn render_loop<F: FnMut(&ash::Device, vk::CommandBuffer, u32)>(&self, mut f: F) {
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
                            f(&self.device, self.draw_command_buffer, framebuffer_index);
                            self.render_end(framebuffer_index);
                        }
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
                .api_version(vk::make_api_version(0, 1, 0, 0));

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
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

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
                pool,
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
                self.device.destroy_command_pool(self.pool, None);
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);
                self.device.destroy_device(None);
                self.surface_loader.destroy_surface(self.surface, None);
                self.instance.destroy_instance(None);
            }
        }
    }
}
