A Vulkan renderer for [yakui](https://github.com/SecondHalfGames/yakui), a declarative UI library for games. Uses [`ash`] to wrap Vulkan related functionality.

The main entrypoint is the [`YakuiVulkan`] struct which creates a [`ash::vk::RenderPass`] and [`ash::vk::Pipeline`] to draw yakui GUIs. This is initialised by populating a [`VulkanContext`] helper struct to pass down the relevant hooks into your Vulkan renderer.

Like most Vulkan applications, this crate uses unsafe Rust! No checks are made to ensure that Vulkan handles are valid, so take note of the safety warnings on the various methods of [`YakuiVulkan`].

Currently this crate only supports drawing to images in the `VK_IMAGE_LAYOUT_PRESENT_SRC_KHR` layout, but future releases will support drawing to any arbitrary [`vk::ImageView`].

This crate requires at least Vulkan 1.2 and a GPU with support for `VkPhysicalDeviceDescriptorIndexingFeatures.descriptorBindingPartiallyBound`. You should also, you know, enable that feature, or Vulkan Validation Layers will get mad at you. You definitely don't want that.

For an example of how to use this crate, check out `examples/demo.rs`