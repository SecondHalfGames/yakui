use ash::vk;

use crate::{descriptors::Descriptors, vulkan_context::VulkanContext};

pub(crate) const NO_TEXTURE_ID: u32 = u32::MAX;

pub(crate) struct VulkanTexture {
    image: vk::Image,
    memory: vk::DeviceMemory,
    pub sampler: vk::Sampler,
    pub view: vk::ImageView,
    pub id: u32,
}

pub struct VulkanTextureCreateInfo<T: AsRef<[u8]>> {
    image_data: T,
    format: vk::Format,
    resolution: vk::Extent2D,
    min_filter: vk::Filter,
    mag_filter: vk::Filter,
}

impl<T: AsRef<[u8]>> VulkanTextureCreateInfo<T> {
    pub fn new(
        image_data: T,
        format: vk::Format,
        resolution: vk::Extent2D,
        min_filter: vk::Filter,
        mag_filter: vk::Filter,
    ) -> Self {
        Self {
            image_data,
            format,
            resolution,
            min_filter,
            mag_filter,
        }
    }
}

impl VulkanTexture {
    pub fn new<T: AsRef<[u8]>>(
        vulkan_context: &VulkanContext,
        descriptors: &mut Descriptors,
        create_info: VulkanTextureCreateInfo<T>,
    ) -> Self {
        let VulkanTextureCreateInfo {
            image_data,
            format,
            resolution,
            min_filter,
            mag_filter,
        } = create_info;

        let address_mode = vk::SamplerAddressMode::REPEAT;
        let (image, memory) =
            unsafe { vulkan_context.create_image(image_data.as_ref(), resolution, format) };
        let view = unsafe { vulkan_context.create_image_view(image, format) };

        let sampler = unsafe {
            vulkan_context
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::builder()
                        .address_mode_u(address_mode)
                        .address_mode_v(address_mode)
                        .address_mode_w(address_mode)
                        .mag_filter(mag_filter)
                        .min_filter(min_filter),
                    None,
                )
                .unwrap()
        };

        let id =
            unsafe { descriptors.update_texture_descriptor_set(view, sampler, vulkan_context) };

        VulkanTexture {
            image,
            memory,
            view,
            sampler,
            id,
        }
    }

    pub fn from_yakui_texture(
        vulkan_context: &VulkanContext,
        descriptors: &mut Descriptors,
        texture: &yakui::paint::Texture,
    ) -> Self {
        let resolution = vk::Extent2D {
            width: texture.size().x,
            height: texture.size().y,
        };

        let format = get_format(texture.format());
        let image_data = texture.data();

        let mag_filter = get_filter(texture.mag_filter);
        let min_filter = get_filter(texture.min_filter);
        VulkanTexture::new(
            vulkan_context,
            descriptors,
            VulkanTextureCreateInfo::new(image_data, format, resolution, min_filter, mag_filter)
        )
    }

    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_sampler(self.sampler, None);
        device.destroy_image_view(self.view, None);
        device.destroy_image(self.image, None);
        device.free_memory(self.memory, None);
    }
}

fn get_format(yakui_format: yakui::paint::TextureFormat) -> vk::Format {
    match yakui_format {
        yakui::paint::TextureFormat::Rgba8Srgb => vk::Format::R8G8B8A8_UNORM,
        yakui::paint::TextureFormat::R8 => vk::Format::R8_UNORM,
        _ => panic!("Unsupported texture format: {yakui_format:?}"),
    }
}

fn get_filter(yakui_filter: yakui::paint::TextureFilter) -> vk::Filter {
    match yakui_filter {
        yakui::paint::TextureFilter::Linear => vk::Filter::LINEAR,
        yakui::paint::TextureFilter::Nearest => vk::Filter::NEAREST,
    }
}
