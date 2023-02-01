use ash::vk;

use crate::{vulkan_context::VulkanContext, vulkan_texture::VulkanTexture};

pub(crate) struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
    texture_count: usize,
}

impl Descriptors {
    pub fn new(vulkan_context: &VulkanContext) -> Descriptors {
        let device = vulkan_context.device;

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::builder()
                    .max_sets(1)
                    .pool_sizes(&[vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1000,
                    }]),
                None,
            )
        }
        .unwrap();

        let flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND];
        let mut binding_flags =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder().binding_flags(&flags);

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::builder()
                    .bindings(&[vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                        descriptor_count: 1000,
                        ..Default::default()
                    }])
                    .push_next(&mut binding_flags),
                None,
            )
        }
        .unwrap();

        let set = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
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

    pub unsafe fn update_texture_descriptor_set(
        &mut self,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        vulkan_context: &VulkanContext,
    ) -> usize {
        let texture_id = self.texture_count;
        vulkan_context.device.update_descriptor_sets(
            std::slice::from_ref(
                &vk::WriteDescriptorSet::builder()
                    .image_info(std::slice::from_ref(
                        &vk::DescriptorImageInfo::builder()
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
