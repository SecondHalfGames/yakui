use ash::vk;

use crate::{util::find_memorytype_index, vulkan_context::VulkanContext};

/// Minimum size of a buffer to avoid needless allocations.
const MIN_BUFFER_SIZE: vk::DeviceSize = 1_048_576; // 1MB

/// A wrapper around a host mapped buffer on the GPU
///
/// May be slightly slow on desktop, but future work could be done to use DEVICE_LOCAL memory if
/// need be.
pub struct Buffer<T> {
    pub handle: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub _usage: vk::BufferUsageFlags,
    /// The maximum size of the buffer, in bytes
    pub size: vk::DeviceSize,
    len: usize,
    ptr: std::ptr::NonNull<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn with_capacity(
        vulkan_context: &VulkanContext,
        usage: vk::BufferUsageFlags,
        elements: usize,
    ) -> Self {
        let device = vulkan_context.device;
        let device_memory_properties = &vulkan_context.memory_properties;

        let buffer_info = vk::BufferCreateInfo::default()
            .size(((std::mem::size_of::<T>() * elements) as vk::DeviceSize).max(MIN_BUFFER_SIZE))
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&buffer_info, None).unwrap() };
        let memory_req = unsafe { device.get_buffer_memory_requirements(handle) };
        let size = MIN_BUFFER_SIZE.max(memory_req.size);
        let memory_index = find_memorytype_index(
            &memory_req,
            device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&allocate_info, None).unwrap() };
        let ptr = unsafe {
            let ptr = device
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();
            device.bind_buffer_memory(handle, memory, 0).unwrap();

            // Safety: ptr is guaranteed to be a non-null pointer to T
            std::ptr::NonNull::new_unchecked(ptr.cast())
        };

        Buffer {
            handle,
            memory,
            _usage: usage,
            size,
            len: elements,
            ptr,
        }
    }

    pub fn new(
        vulkan_context: &VulkanContext,
        usage: vk::BufferUsageFlags,
        initial_data: &[T],
    ) -> Self {
        let mut result = Self::with_capacity(vulkan_context, usage, initial_data.len());
        unsafe {
            result.write(vulkan_context, 0, initial_data);
        }
        result
    }

    pub unsafe fn write(&mut self, _vulkan_context: &VulkanContext, offset: usize, new_data: &[T]) {
        let new_data_size = std::mem::size_of_val(new_data);
        if std::mem::size_of::<T>() * offset + new_data_size > self.size as usize {
            todo!("Support resizing buffers");
        }

        self.ptr
            .as_ptr()
            .add(offset)
            .copy_from_nonoverlapping(new_data.as_ptr(), new_data.len());
    }

    /// ## Safety
    /// Buffer will be unusable after this function has been called.
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.unmap_memory(self.memory);
        device.free_memory(self.memory, None);
        device.destroy_buffer(self.handle, None);
    }

    pub fn capacity(&self) -> usize {
        self.len
    }
}
