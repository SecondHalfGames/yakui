use ash::vk;

use crate::{descriptors::Descriptors, vulkan_context::VulkanContext};

pub(crate) struct VulkanTexture {
    resolution: vk::Extent2D,
    yakui_format: yakui_core::paint::TextureFormat,
    format: vk::Format,
    image: vk::Image,
    pub sampler: vk::Sampler,
    pub view: vk::ImageView,
    pub id: usize,
}

impl VulkanTexture {
    pub fn new(
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

        let image = unsafe { vulkan_context.create_image(image_data, resolution, format) };
        let view = unsafe { vulkan_context.create_image_view(image, format) };

        let address_mode = vk::SamplerAddressMode::REPEAT;
        let mag_filter = get_filter(texture.mag_filter);
        let min_filter = get_filter(texture.min_filter);

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
            resolution,
            yakui_format: texture.format(),
            format,
            image,
            view,
            sampler,
            id,
        }
    }
}

fn get_format(yakui_format: yakui::paint::TextureFormat) -> vk::Format {
    match yakui_format {
        yakui::paint::TextureFormat::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
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
