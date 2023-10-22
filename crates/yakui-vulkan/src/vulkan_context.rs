use ash::vk;

use crate::util::find_memorytype_index;

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
    /// Memory properties used for [`crate::YakuiVulkan`]'s allocation commands
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl<'a> VulkanContext<'a> {
    /// Construct a new [`VulkanContext`] wrapper.
    pub fn new(
        device: &'a ash::Device,
        queue: vk::Queue,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        Self {
            device,
            queue,
            memory_properties,
        }
    }

    pub(crate) unsafe fn create_image(
        &self,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> (vk::Image, vk::DeviceMemory) {
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

        (image, image_memory)
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
