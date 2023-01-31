use ash::vk;

use crate::vulkan_context::VulkanContext;

pub struct VulkanTexture {
    resolution: vk::Extent2D,
    yakui_format: yakui_core::paint::TextureFormat,
    format: vk::Format,
    image: vk::Image,
    pub view: vk::ImageView,
}

impl VulkanTexture {
    pub fn new(vulkan_context: &VulkanContext, texture: &yakui::paint::Texture) -> Self {
        let resolution = vk::Extent2D {
            width: texture.size().x,
            height: texture.size().y,
        };

        let format = get_format(texture.format());
        let image_data = texture.data();

        let image = unsafe { vulkan_context.create_image(image_data, resolution, format) };
        let view = unsafe { vulkan_context.create_image_view(image, format) };

        VulkanTexture {
            resolution,
            yakui_format: texture.format(),
            format,
            image,
            view,
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
