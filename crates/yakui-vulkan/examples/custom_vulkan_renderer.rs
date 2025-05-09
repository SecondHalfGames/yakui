use std::time::Instant;

use ash::vk;
use bytemuck::bytes_of;
use thunderdome::{Arena, Index};
use yakui::paint::{PaintCall, UserPaintCallId};
use yakui::util::widget;
use yakui::widget::Widget;
use yakui::{Color, Rect, UVec2, Vec2};
use yakui_vulkan::*;

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
struct MyCustomRenderer {
    objects: Arena<MyCustomRenderedObject>,
    initial_time: Instant,
}

impl MyCustomRenderer {
    pub fn add(&mut self, object: MyCustomRenderedObject) -> UserPaintCallId {
        self.objects.insert(object).to_bits()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn new() -> Self {
        Self {
            objects: Arena::new(),
            initial_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MyCustomRenderedObject {
    color: Color,
}

impl MyCustomRenderedObject {
    pub fn show(self) -> yakui::Response<()> {
        widget::<MyCustomRenderedWidget>(self)
    }
}

#[derive(Debug)]
struct MyCustomRenderedWidget {
    props: MyCustomRenderedObject,
}

impl Widget for MyCustomRenderedWidget {
    type Props<'a> = MyCustomRenderedObject;

    type Response = ();

    fn new() -> Self {
        Self {
            props: MyCustomRenderedObject {
                color: Color::CLEAR,
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props
    }

    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        _constraints: yakui::Constraints,
    ) -> yakui::Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        yakui::Vec2::new(80., 80.)
    }

    fn paint(&self, ctx: yakui::widget::PaintContext<'_>) {
        let id = ctx
            .paint
            .globals
            .get_mut(MyCustomRenderer::new)
            .add(self.props);

        ctx.paint.add_user_call(id);
    }
}

unsafe fn my_custom_paint(
    yakui_vulkan: &mut YakuiVulkan,
    paint: &mut yakui_core::paint::PaintDom,
    vulkan_context: &VulkanContext,
    cmd: vk::CommandBuffer,
    resolution: vk::Extent2D,
) {
    // --- yakui ---
    // If there's nothing to paint, well... don't paint!
    let layers = &paint.layers;
    if layers.iter().all(|layer| layer.calls.is_empty()) {
        return;
    }

    // If the surface has a size of zero, well... don't paint either!
    if paint.surface_size().x == 0.0 || paint.surface_size().y == 0.0 {
        return;
    }

    let mut vertices: Vec<Vertex> = Default::default();
    let mut indices: Vec<u32> = Default::default();
    let mut draw_calls: Vec<(Rect, DrawCall)> = Default::default();
    // --- yakui ---

    let renderer = paint.globals.get_mut(MyCustomRenderer::new);

    let mut custom_draws = Arena::new();
    let elapsed = Instant::now()
        .duration_since(renderer.initial_time)
        .as_secs_f32();

    for (clip, call) in layers.iter().flat_map(|layer| &layer.calls) {
        match call {
            PaintCall::Internal(call) => {
                draw_calls.push(yakui_vulkan.build_draw_call(
                    &mut vertices,
                    &mut indices,
                    *clip,
                    call,
                ));
            }
            PaintCall::User(id) => {
                let min = clip.pos();
                let max = clip.max();

                let x1y1 = min;
                let x1y2 = Vec2::new(min.x, max.y);
                let x2y1 = Vec2::new(max.x, min.y);
                let x2y2 = max;

                let index = Index::from_bits(*id).unwrap();
                let object = renderer.objects.get(index).unwrap();
                let mut color = object.color.to_linear();

                color.x = (color.x + elapsed).sin().abs();
                color.y = (color.y + elapsed).cos().abs();

                let my_indices = [0, 1, 2, 1, 3, 2];

                let base = vertices.len() as u32;
                let index_offset = indices.len() as u32;
                let index_count = my_indices.len() as u32;

                for i in my_indices {
                    indices.push(i as u32 + base);
                }

                for pos in [x1y1, x1y2, x2y1, x2y2] {
                    vertices.push(Vertex {
                        position: paint.info.transform_vertex(pos, false),
                        texcoord: Vec2::default(),
                        color,
                    });
                }

                draw_calls.push((*clip, DrawCall::User(*id)));
                custom_draws.insert_at(index, (index_offset, index_count));
            }
        }
    }

    renderer.clear();

    // --- yakui ---
    unsafe {
        yakui_vulkan.index_buffer.write(vulkan_context, 0, &indices);
        yakui_vulkan
            .vertex_buffer
            .write(vulkan_context, 0, &vertices);
    }

    let device = vulkan_context.device;
    let surface_size = UVec2::new(resolution.width, resolution.height);
    // --- yakui ---

    unsafe {
        // --- yakui ---
        device.cmd_bind_pipeline(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            yakui_vulkan.graphics_pipeline,
        );
        let default_scissor = [resolution.into()];
        device.cmd_set_scissor(cmd, 0, &default_scissor);

        device.cmd_bind_vertex_buffers(cmd, 0, &[yakui_vulkan.vertex_buffer.handle], &[0]);
        device.cmd_bind_index_buffer(
            cmd,
            yakui_vulkan.index_buffer.handle,
            0,
            vk::IndexType::UINT32,
        );
        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            yakui_vulkan.pipeline_layout,
            0,
            std::slice::from_ref(&yakui_vulkan.descriptors.set),
            &[],
        );

        let mut last_clip = None;

        for (clip, draw_call) in draw_calls {
            if Some(clip) != last_clip {
                last_clip = Some(clip);

                // TODO - do this when processing draw calls
                let pos = clip.pos().as_uvec2();
                let size = clip.size().as_uvec2();

                let max = (pos + size).min(surface_size);
                let size = UVec2::new(max.x.saturating_sub(pos.x), max.y.saturating_sub(pos.y));

                // If the scissor rect isn't valid, we can skip this
                // entire draw call.
                if pos.x > surface_size.x || pos.y > surface_size.y || size.x == 0 || size.y == 0 {
                    continue;
                }

                let scissors = [vk::Rect2D {
                    offset: vk::Offset2D {
                        x: pos.x as _,
                        y: pos.y as _,
                    },
                    extent: vk::Extent2D {
                        width: size.x,
                        height: size.y,
                    },
                }];
                // If there's a clip, update the scissor
                device.cmd_set_scissor(cmd, 0, &scissors);
            }
            // --- yakui ---

            match draw_call {
                DrawCall::Yakui(draw_call) => {
                    yakui_vulkan.draw_yakui(vulkan_context, cmd, draw_call);
                }
                DrawCall::User(id) => {
                    device.cmd_push_constants(
                        cmd,
                        yakui_vulkan.pipeline_layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytes_of(&PushConstant::new(NO_TEXTURE_ID, Workflow::Main)),
                    );

                    let (index_offset, index_count) =
                        custom_draws.remove(Index::from_bits(id).unwrap()).unwrap();

                    device.cmd_draw_indexed(cmd, index_count, 1, index_offset, 0, 1);
                }
            }
        }
    }
}

/// Simple test to make sure Vulkan backend renders properly with a custom renderer.
fn main() {
    use winit::dpi::PhysicalSize;

    let (width, height) = (500, 500);
    let (event_loop, window) = init_winit(width, height);
    let mut vulkan_test = VulkanTest::new(width, height, &window);

    let mut yak = yakui::Yakui::new();
    yak.set_surface_size([width as f32, height as f32].into());
    yak.set_unscaled_viewport(Rect::from_pos_size(
        Default::default(),
        [width as f32, height as f32].into(),
    ));

    let mut yakui_vulkan = {
        let vulkan_context = VulkanContext::new(
            &vulkan_test.device,
            vulkan_test.present_queue,
            vulkan_test.device_memory_properties,
            vulkan_test.device_properties,
        );
        let options = yakui_vulkan::Options {
            render_pass: vulkan_test.render_pass,
            ..Default::default()
        };
        let mut yakui_vulkan = YakuiVulkan::new(&vulkan_context, options);
        yakui_vulkan.set_paint_limits(&vulkan_context, &mut yak);
        // Prepare for one frame in flight
        yakui_vulkan.transfers_submitted();
        yakui_vulkan
    };

    let mut winit_initializing = true;

    event_loop.set_control_flow(ControlFlow::Poll);
    #[allow(deprecated)] // winit!! :shakes-fist:
    let _ = event_loop.run(|event, event_loop| match event {
        Event::WindowEvent {
            event:
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                },
            ..
        } => event_loop.exit(),

        Event::NewEvents(cause) => {
            winit_initializing = cause == winit::event::StartCause::Init;
        }

        Event::AboutToWait => {
            let vulkan_context = VulkanContext::new(
                &vulkan_test.device,
                vulkan_test.present_queue,
                vulkan_test.device_memory_properties,
                vulkan_test.device_properties,
            );

            yak.start();
            gui();
            yak.finish();

            let paint = yak.paint();

            let index = vulkan_test.cmd_begin();
            unsafe {
                yakui_vulkan.transfers_finished(&vulkan_context);
                yakui_vulkan.transfer(paint, &vulkan_context, vulkan_test.draw_command_buffer);
            }
            vulkan_test.render_begin(index);
            unsafe {
                my_custom_paint(
                    &mut yakui_vulkan,
                    paint,
                    &vulkan_context,
                    vulkan_test.draw_command_buffer,
                    vulkan_test.swapchain_info.surface_resolution,
                );
            }

            vulkan_test.render_end(index);
            yakui_vulkan.transfers_submitted();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            if winit_initializing {
                println!("Ignoring resize during init!");
            } else {
                let PhysicalSize { width, height } = size;
                vulkan_test.resized(width, height);
                yak.set_surface_size([width as f32, height as f32].into());
                yak.set_unscaled_viewport(Rect::from_pos_size(
                    Default::default(),
                    [width as f32, height as f32].into(),
                ));
            }
        }
        Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
            ..
        } => yak.set_scale_factor(scale_factor as _),
        _ => (),
    });

    unsafe {
        yakui_vulkan.cleanup(&vulkan_test.device);
    }
}

fn gui() {
    use yakui::{column, label, row, widgets::Text, Color};
    column(|| {
        row(|| {
            label("Hello, world!");

            let mut text = Text::new(48.0, "colored text!");
            text.style.color = Color::RED;
            text.show();
        });

        row(|| {
            MyCustomRenderedObject { color: Color::RED }.show();
            MyCustomRenderedObject {
                color: Color::GREEN,
            }
            .show();
            MyCustomRenderedObject { color: Color::BLUE }.show();
            MyCustomRenderedObject {
                color: Color::YELLOW,
            }
            .show();
            MyCustomRenderedObject {
                color: Color::WHITE,
            }
            .show();
            MyCustomRenderedObject {
                color: Color::FUCHSIA,
            }
            .show();
        });
    });
}

struct VulkanTest {
    _entry: ash::Entry,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    instance: ash::Instance,
    surface_loader: ash::khr::surface::Instance,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    device_properties: vk::PhysicalDeviceProperties,

    present_queue: vk::Queue,

    surface: vk::SurfaceKHR,
    swapchain_info: SwapchainInfo,

    swapchain: vk::SwapchainKHR,
    present_image_views: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    draw_command_buffer: vk::CommandBuffer,

    present_complete_semaphore: vk::Semaphore,
    rendering_complete_semaphore: vk::Semaphore,

    draw_commands_reuse_fence: vk::Fence,
    setup_commands_reuse_fence: vk::Fence,
}

impl VulkanTest {
    /// Bring up all the Vulkan pomp and ceremony required to render things.
    /// Vulkan Broadly lifted from: https://github.com/ash-rs/ash/blob/0.37.2/examples/src/lib.rs
    pub fn new(window_width: u32, window_height: u32, window: &winit::window::Window) -> Self {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

        let entry = unsafe { ash::Entry::load().expect("failed to load Vulkan") };
        let app_name = c"Yakui Vulkan Test";

        let appinfo = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 3, 0));

        #[allow(unused_mut)]
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();

        #[cfg(target_os = "macos")]
        extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());

        let create_flags = if cfg!(target_os = "macos") {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let create_info = vk::InstanceCreateInfo::default()
            .flags(create_flags)
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
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        };

        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let (physical_device, queue_family_index) = unsafe {
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

        #[allow(unused_mut)]
        let mut device_exts = vec![ash::khr::swapchain::NAME.as_ptr()];

        #[cfg(target_os = "macos")]
        device_exts.push(ash::khr::portability_subset::NAME.as_ptr());

        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let mut descriptor_indexing_features =
            vk::PhysicalDeviceDescriptorIndexingFeatures::default()
                .descriptor_binding_partially_bound(true)
                .descriptor_binding_sampled_image_update_after_bind(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_exts)
            .push_next(&mut descriptor_indexing_features);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap()
        };

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
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

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        let swapchain_info = SwapchainInfo::new(
            swapchain_loader,
            surface_format,
            surface_resolution,
            present_mode,
            surface,
            desired_image_count,
        );

        let (swapchain, present_image_views) = create_swapchain(&device, None, &swapchain_info);

        let renderpass_attachments = [vk::AttachmentDescription {
            format: swapchain_info.surface_format.format,
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

        let subpass = vk::SubpassDescription::default()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let framebuffers = create_framebuffers(
            &present_image_views,
            surface_resolution,
            render_pass,
            &device,
        );

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()
        };
        let draw_command_buffer = command_buffers[0];

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

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
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

        Self {
            device,
            physical_device,
            present_queue,
            _entry: entry,
            instance,
            surface_loader,
            swapchain_info,
            device_memory_properties,
            device_properties,
            surface,
            swapchain,
            present_image_views,
            render_pass,
            framebuffers,
            command_pool: pool,
            draw_command_buffer,
            present_complete_semaphore,
            rendering_complete_semaphore,
            draw_commands_reuse_fence,
            setup_commands_reuse_fence,
        }
    }

    pub fn resized(&mut self, window_width: u32, window_height: u32) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            let surface_capabilities = self
                .surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .unwrap();
            let surface_resolution = match surface_capabilities.current_extent.width {
                std::u32::MAX => vk::Extent2D {
                    width: window_width,
                    height: window_height,
                },
                _ => surface_capabilities.current_extent,
            };
            self.swapchain_info.surface_resolution = surface_resolution;
            let (new_swapchain, new_present_image_views) =
                create_swapchain(&self.device, Some(self.swapchain), &self.swapchain_info);
            let framebuffers = create_framebuffers(
                &new_present_image_views,
                self.swapchain_info.surface_resolution,
                self.render_pass,
                &self.device,
            );

            self.destroy_swapchain(self.swapchain);
            self.present_image_views = new_present_image_views;
            self.swapchain = new_swapchain;
            self.framebuffers = framebuffers;
        }
    }

    unsafe fn destroy_swapchain(&self, swapchain: vk::SwapchainKHR) {
        let device = &self.device;
        for &fb in &self.framebuffers {
            device.destroy_framebuffer(fb, None);
        }
        for image_view in &self.present_image_views {
            device.destroy_image_view(*image_view, None);
        }
        self.swapchain_info
            .swapchain_loader
            .destroy_swapchain(swapchain, None);
    }

    pub fn cmd_begin(&self) -> u32 {
        let (present_index, _) = unsafe {
            self.swapchain_info
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
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
                    u64::MAX,
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
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap();
        }
        present_index
    }

    pub fn render_begin(&self, present_index: u32) -> u32 {
        let device = &self.device;
        unsafe {
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
            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain_info.surface_resolution.width as f32,
                height: self.swapchain_info.surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[present_index as usize])
                .render_area(self.swapchain_info.surface_resolution.into())
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(
                self.draw_command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_set_viewport(self.draw_command_buffer, 0, &viewports);
            device.cmd_set_scissor(
                self.draw_command_buffer,
                0,
                &[self.swapchain_info.surface_resolution.into()],
            );
        }
        present_index
    }

    pub fn render_end(&self, present_index: u32) {
        let device = &self.device;
        unsafe {
            device.cmd_end_render_pass(self.draw_command_buffer);
            device.end_command_buffer(self.draw_command_buffer).unwrap();
            let swapchains = [self.swapchain];
            let image_indices = [present_index];
            let submit_info = vk::SubmitInfo::default()
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

            match self.swapchain_info.swapchain_loader.queue_present(
                self.present_queue,
                &vk::PresentInfoKHR::default()
                    .image_indices(&image_indices)
                    .wait_semaphores(std::slice::from_ref(&self.rendering_complete_semaphore))
                    .swapchains(&swapchains),
            ) {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // this usually indicates the window has been resized
                }
                Err(e) => panic!("Error presenting: {e:?}"),
                _ => {}
            }
        };
    }
}

fn init_winit(
    window_width: u32,
    window_height: u32,
) -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
    use winit::{event_loop::EventLoop, window::Window};

    let event_loop = EventLoop::new().unwrap();

    #[allow(deprecated)]
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Yakui Vulkan - Test")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    f64::from(window_width),
                    f64::from(window_height),
                )),
        )
        .unwrap();
    (event_loop, window)
}

fn create_swapchain(
    device: &ash::Device,
    previous_swapchain: Option<vk::SwapchainKHR>,
    swapchain_info: &SwapchainInfo,
) -> (vk::SwapchainKHR, Vec<vk::ImageView>) {
    let SwapchainInfo {
        swapchain_loader,
        surface_format,
        surface_resolution,
        present_mode,
        surface,
        desired_image_count,
    } = swapchain_info;

    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(*surface)
        .min_image_count(*desired_image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(*surface_resolution)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(*present_mode)
        .clipped(true)
        .image_array_layers(1);

    if let Some(old_swapchain) = previous_swapchain {
        swapchain_create_info.old_swapchain = old_swapchain
    }

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .unwrap()
    };

    let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
    let present_image_views: Vec<vk::ImageView> = present_images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo::default()
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

    (swapchain, present_image_views)
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
            self.device.destroy_command_pool(self.command_pool, None);
            self.destroy_swapchain(self.swapchain);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

struct SwapchainInfo {
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,
    pub present_mode: vk::PresentModeKHR,
    pub surface: vk::SurfaceKHR,
    pub desired_image_count: u32,
}

impl SwapchainInfo {
    pub fn new(
        swapchain_loader: ash::khr::swapchain::Device,
        surface_format: vk::SurfaceFormatKHR,
        surface_resolution: vk::Extent2D,
        present_mode: vk::PresentModeKHR,
        surface: vk::SurfaceKHR,
        desired_image_count: u32,
    ) -> Self {
        Self {
            swapchain_loader,
            surface_format,
            surface_resolution,
            present_mode,
            surface,
            desired_image_count,
        }
    }
}

fn create_framebuffers(
    views: &[vk::ImageView],
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    device: &ash::Device,
) -> Vec<vk::Framebuffer> {
    let framebuffers: Vec<vk::Framebuffer> = views
        .iter()
        .map(|&present_image_view| {
            let framebuffer_attachments = [present_image_view];
            let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&framebuffer_attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            unsafe {
                device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            }
        })
        .collect();
    framebuffers
}
