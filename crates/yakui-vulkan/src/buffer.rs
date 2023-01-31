use std::mem::align_of;

use ash::{util::Align, vk};

use crate::{util::find_memorytype_index, vulkan_context::VulkanContext};

/// Minimum size of a buffer to avoid needless allocations.
const MIN_BUFFER_SIZE: vk::DeviceSize = 1_048_576; // 1MB

/// A wrapper around a host mapped buffer on the GPU
///
/// May be slightly slow on desktop, but future work could be done to use DEVICE_LOCAL memory if
/// need be.
pub(crate) struct Buffer<T> {
    pub handle: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub _usage: vk::BufferUsageFlags,
    /// The maximum size of the buffer, in bytes
    pub size: vk::DeviceSize,
    len: usize,
    ptr: std::ptr::NonNull<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(
        vulkan_context: &VulkanContext,
        usage: vk::BufferUsageFlags,
        initial_data: &[T],
    ) -> Self {
        let device = vulkan_context.device;
        let device_memory_properties = &vulkan_context.memory_properties;

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(initial_data).max(MIN_BUFFER_SIZE as _) as u64)
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
            let mut slice = Align::new(ptr, align_of::<T>() as u64, size);
            slice.copy_from_slice(initial_data);
            device.bind_buffer_memory(handle, memory, 0).unwrap();

            // Safety: ptr is guaranteed to be a non-null pointer to T
            std::ptr::NonNull::new_unchecked(ptr.cast())
        };

        Buffer {
            handle,
            memory,
            _usage: usage,
            size,
            len: initial_data.len(),
            ptr,
        }
    }

    /// The current size of the buffer, in bytes
    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn overwrite(&mut self, _vulkan_context: &VulkanContext, new_data: &[T]) {
        let new_data_size = std::mem::size_of_val(new_data);
        if new_data_size > self.size as usize {
            todo!("Support resizing buffers");
        }

        let mut slice = Align::new(self.ptr.as_ptr().cast(), align_of::<T>() as u64, self.size);
        slice.copy_from_slice(new_data);

        self.len = new_data.len()
    }

    // TODO: clippy will whinge about this
    pub fn _is_empty(&self) -> bool {
        self.len == 0
    }

    /// ## Safety
    /// Buffer will be unusable after this function has been called.
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.unmap_memory(self.memory);
        device.free_memory(self.memory, None);
        device.destroy_buffer(self.handle, None);
    }
}
