use ash::vk;

pub(crate) fn find_memorytype_index(
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

#[cfg(test)]
pub(crate) struct VulkanTest {
    pub _entry: ash::Entry,
    pub device: ash::Device,
    pub instance: ash::Instance,
    pub surface_loader: ash::extensions::khr::Surface,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,

    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub swapchain_info: SwapchainInfo,

    pub swapchain: vk::SwapchainKHR,
    pub present_image_views: Vec<vk::ImageView>,

    pub command_pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,
}

#[cfg(test)]
impl VulkanTest {
    /// Bring up all the Vulkan pomp and ceremony required to render things.
    /// Vulkan Broadly lifted from: https://github.com/ash-rs/ash/blob/0.37.2/examples/src/lib.rs
    pub fn new(window_width: u32, window_height: u32, window: &winit::window::Window) -> Self {
        use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
        use std::ffi::CStr;

        let entry = unsafe { ash::Entry::load().expect("failed to load Vulkan") };
        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"Yakui Vulkan Test\0") };

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
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
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
        let device_extension_names_raw = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let mut descriptor_indexing_features =
            vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
                .descriptor_binding_partially_bound(true);

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
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
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);

        let swapchain_info = SwapchainInfo::new(
            swapchain_loader,
            surface_format,
            surface_resolution,
            present_mode,
            surface,
            desired_image_count,
        );

        let (swapchain, present_image_views) = create_swapchain(&device, None, &swapchain_info);

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
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        Self {
            device,
            present_queue,
            _entry: entry,
            instance,
            surface_loader,
            swapchain_info,
            device_memory_properties,
            surface,
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

    pub fn resized(&mut self, window_width: u32, window_height: u32) {
        println!("Vulkan Resized: {window_width}, {window_height}");
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.swapchain_info.surface_resolution = vk::Extent2D {
                width: window_width,
                height: window_height,
            };
            let (new_swapchain, new_present_image_views) =
                create_swapchain(&self.device, Some(self.swapchain), &self.swapchain_info);

            self.destroy_swapchain(self.swapchain);
            self.present_image_views = new_present_image_views;
            self.swapchain = new_swapchain;
        }
        println!("OK! Swapchain recreated");
    }

    unsafe fn destroy_swapchain(&self, swapchain: vk::SwapchainKHR) {
        let device = &self.device;
        for image_view in &self.present_image_views {
            device.destroy_image_view(*image_view, None);
        }
        self.swapchain_info
            .swapchain_loader
            .destroy_swapchain(swapchain, None);
    }

    pub fn render_begin(&self) -> u32 {
        let (present_index, _) = unsafe {
            self.swapchain_info
                .swapchain_loader
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

    pub fn render_end(&self, present_index: u32) {
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

            match self.swapchain_info.swapchain_loader.queue_present(
                self.present_queue,
                &vk::PresentInfoKHR::builder()
                    .image_indices(&image_indices)
                    .wait_semaphores(std::slice::from_ref(&self.rendering_complete_semaphore))
                    .swapchains(&swapchains),
            ) {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    println!("Swapchain is suboptimal!")
                }
                Err(e) => panic!("Error presenting: {e:?}"),
                _ => {}
            }
        };
    }
}

#[cfg(all(windows, test))]
pub(crate) fn init_winit(
    window_width: u32,
    window_height: u32,
) -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
    use winit::{
        event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
        window::WindowBuilder,
    };

    // necessary because tests are in a separate thread
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    let window = WindowBuilder::new()
        .with_title("Yakui Vulkan - Test")
        .with_inner_size(winit::dpi::LogicalSize::new(
            f64::from(window_width),
            f64::from(window_height),
        ))
        .build(&event_loop)
        .unwrap();
    (event_loop, window)
}

#[cfg(test)]
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

    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
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

    (swapchain, present_image_views)
}

#[cfg(test)]
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
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

#[cfg(test)]
pub(crate) struct SwapchainInfo {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,
    pub present_mode: vk::PresentModeKHR,
    pub surface: vk::SurfaceKHR,
    pub desired_image_count: u32,
}

#[cfg(test)]
impl SwapchainInfo {
    pub fn new(
        swapchain_loader: ash::extensions::khr::Swapchain,
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
