use ash::vk;

use crate::{buffer::Buffer, util::find_memorytype_index};

#[derive(Clone)]
/// A wrapper around handles into your Vulkan renderer to call various methods on [`crate::YakuiVulkan`]
///
/// ## Safety
/// It is **very** important that you pass the correct handles to this struct, or you will have a terrible time.
/// Once you create a [`crate::YakuiVulkan`] instance, you **must** use the same handles each time you call a
/// method on that instance.
///
/// See the documentation on each member for specific safety requirements.
pub struct VulkanContext<'a> {
    /// A valid Vulkan device (see the [crate level documentation](`crate`) for specifics)
    pub device: &'a ash::Device,
    /// A queue that can call render and transfer commands
    pub queue: vk::Queue,
    /// The command buffer that you'll ultimately submit to be presented/rendered
    pub draw_command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    /// Memory properties used for [`crate::YakuiVulkan`]'s allocation commands
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl<'a> VulkanContext<'a> {
    /// Construct a new [`VulkanContext`] wrapper.
    pub fn new(
        device: &'a ash::Device,
        queue: vk::Queue,
        draw_command_buffer: vk::CommandBuffer,
        command_pool: vk::CommandPool,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        Self {
            device,
            queue,
            draw_command_buffer,
            command_pool,
            memory_properties,
        }
    }

    pub(crate) unsafe fn create_image(
        &self,
        image_data: &[u8],
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> (vk::Image, vk::DeviceMemory) {
        let scratch_buffer = Buffer::new(self, vk::BufferUsageFlags::TRANSFER_SRC, image_data);
        let device = self.device;

        let image = device
            .create_image(
                &vk::ImageCreateInfo {
                    image_type: vk::ImageType::TYPE_2D,
                    format,
                    extent: extent.into(),
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SampleCountFlags::TYPE_1,
                    tiling: vk::ImageTiling::OPTIMAL,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    sharing_mode: vk::SharingMode::EXCLUSIVE,
                    ..Default::default()
                },
                None,
            )
            .unwrap();

        let memory_requirements = device.get_image_memory_requirements(image);
        let memory_index = find_memorytype_index(
            &memory_requirements,
            &self.memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory type for image");
        let image_memory = self.allocate_memory(memory_requirements.size, memory_index);
        device.bind_image_memory(image, image_memory, 0).unwrap();

        self.one_time_command(|command_buffer| {
            let transfer_barrier = vk::ImageMemoryBarrier {
                dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[transfer_barrier],
            );
            let buffer_copy_regions = vk::BufferImageCopy {
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    layer_count: 1,
                    ..Default::default()
                },
                image_extent: extent.into(),
                ..Default::default()
            };

            device.cmd_copy_buffer_to_image(
                command_buffer,
                scratch_buffer.handle,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&buffer_copy_regions),
            );

            let transition_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&transition_barrier),
            )
        });

        scratch_buffer.cleanup(device);

        (image, image_memory)
    }

    unsafe fn one_time_command<F: FnOnce(vk::CommandBuffer)>(&self, f: F) {
        let device = &self.device;
        let command_buffer = device
            .allocate_command_buffers(&vk::CommandBufferAllocateInfo {
                command_pool: self.command_pool,
                command_buffer_count: 1,
                level: vk::CommandBufferLevel::PRIMARY,
                ..Default::default()
            })
            .unwrap()[0];

        let fence = device.create_fence(&Default::default(), None).unwrap();

        device
            .begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo {
                    flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    ..Default::default()
                },
            )
            .unwrap();

        f(command_buffer);

        device.end_command_buffer(command_buffer).unwrap();

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));
        device
            .queue_submit(self.queue, std::slice::from_ref(&submit_info), fence)
            .unwrap();
        device
            .wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX)
            .unwrap();

        device.destroy_fence(fence, None);
        device.free_command_buffers(self.command_pool, std::slice::from_ref(&command_buffer));
    }

    unsafe fn allocate_memory(
        &self,
        allocation_size: vk::DeviceSize,
        memory_type_index: u32,
    ) -> vk::DeviceMemory {
        self.device
            .allocate_memory(
                &vk::MemoryAllocateInfo {
                    allocation_size,
                    memory_type_index,
                    ..Default::default()
                },
                None,
            )
            .unwrap()
    }

    pub(crate) unsafe fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
    ) -> vk::ImageView {
        self.device
            .create_image_view(
                &vk::ImageViewCreateInfo {
                    image,
                    format,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    },
                    view_type: vk::ImageViewType::TYPE_2D,
                    ..Default::default()
                },
                None,
            )
            .unwrap()
    }
}
