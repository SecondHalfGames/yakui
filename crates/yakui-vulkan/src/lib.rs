use ash::vk;

pub struct YakuiVulkan {}

impl YakuiVulkan {
    pub fn new(device: &ash::Device) -> Self {
        Self {}
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
        let vulkan_test = VulkanTest::new(500, 500);
        let yakui_vulkan = YakuiVulkan::new(&vulkan_test.device);
    }

    struct VulkanTest {
        pub window: winit::window::Window,
        pub device: ash::Device,
        pub entry: ash::Entry,
        pub instance: ash::Instance,
        pub surface_loader: Surface,
        pub swapchain_loader: Swapchain,
        pub event_loop: RefCell<winit::event_loop::EventLoop<()>>,

        pub pdevice: vk::PhysicalDevice,
        pub queue_family_index: u32,
        pub present_queue: vk::Queue,

        pub surface: vk::SurfaceKHR,
        pub surface_format: vk::SurfaceFormatKHR,
        pub surface_resolution: vk::Extent2D,

        pub swapchain: vk::SwapchainKHR,
        pub present_images: Vec<vk::Image>,
        pub present_image_views: Vec<vk::ImageView>,

        pub pool: vk::CommandPool,
        pub draw_command_buffer: vk::CommandBuffer,
        pub setup_command_buffer: vk::CommandBuffer,

        pub present_complete_semaphore: vk::Semaphore,
        pub rendering_complete_semaphore: vk::Semaphore,

        pub draw_commands_reuse_fence: vk::Fence,
        pub setup_commands_reuse_fence: vk::Fence,
    }

    impl VulkanTest {
        pub fn render_loop<F: Fn()>(&self, f: F) {
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
                        Event::MainEventsCleared => f(),
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
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = unsafe {
                device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .unwrap()
            };
            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

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

            Self {
                window,
                device,
                present_queue,
                entry,
                instance,
                surface_loader,
                swapchain_loader,
                event_loop: RefCell::new(event_loop),
                pdevice,
                queue_family_index,
                surface,
                surface_format,
                surface_resolution,
                swapchain,
                present_images,
                present_image_views,
                pool,
                draw_command_buffer,
                setup_command_buffer,
                present_complete_semaphore,
                rendering_complete_semaphore,
                draw_commands_reuse_fence,
                setup_commands_reuse_fence,
            }
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
