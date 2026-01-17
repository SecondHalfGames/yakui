use ash::vk;

use crate::vulkan_context::VulkanContext;

/// A thin wrapper around descriptor related functionality
#[allow(missing_docs)]
pub struct Descriptors {
    pub(crate) pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
    texture_count: u32,
}

impl Descriptors {
    pub(crate) fn new(vulkan_context: &VulkanContext) -> Descriptors {
        let device = vulkan_context.device;

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
                    .pool_sizes(&[vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1000,
                    }]),
                None,
            )
        }
        .unwrap();

        let flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND];
        let mut binding_flags =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default()
                    .bindings(&[vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                        descriptor_count: 1000,
                        ..Default::default()
                    }])
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .push_next(&mut binding_flags),
                None,
            )
        }
        .unwrap();

        let set = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(std::slice::from_ref(&layout)),
            )
        }
        .unwrap()[0];

        Descriptors {
            pool,
            set,
            layout,
            texture_count: 0,
        }
    }

    pub(crate) unsafe fn update_texture_descriptor_set(
        &mut self,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        vulkan_context: &VulkanContext,
    ) -> u32 {
        let texture_id = self.texture_count;
        vulkan_context.device.update_descriptor_sets(
            std::slice::from_ref(
                &vk::WriteDescriptorSet::default()
                    .image_info(std::slice::from_ref(
                        &vk::DescriptorImageInfo::default()
                            .sampler(sampler)
                            .image_view(image_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
                    ))
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .dst_array_element(texture_id as _)
                    .dst_set(self.set),
            ),
            &[],
        );

        self.texture_count += 1;
        texture_id
    }

    /// ## Safety
    /// Descriptors will be unusable after this function has been called.
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_descriptor_set_layout(self.layout, None);
        device.destroy_descriptor_pool(self.pool, None);
    }
}
