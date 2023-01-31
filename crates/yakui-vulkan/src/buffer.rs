use std::{marker::PhantomData, mem::align_of};

use ash::{util::Align, vk};

use crate::{util::find_memorytype_index, vulkan_context::VulkanContext};

pub(crate) struct Buffer<T> {
    pub handle: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub _usage: vk::BufferUsageFlags,
    pub _max_size: vk::DeviceSize,
    len: usize,
    _phantom: PhantomData<T>,
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
            .size(std::mem::size_of_val(initial_data) as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&buffer_info, None).unwrap() };
        let memory_req = unsafe { device.get_buffer_memory_requirements(handle) };
        let max_size = memory_req.size;
        let memory_index = find_memorytype_index(
            &memory_req,
            device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: max_size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&allocate_info, None).unwrap() };
        unsafe {
            upload(device, memory, max_size, initial_data);
            device.bind_buffer_memory(handle, memory, 0).unwrap();
        }

        Buffer {
            handle,
            memory,
            _usage: usage,
            _max_size: max_size,
            len: initial_data.len(),
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn _overwrite(&mut self, device: &ash::Device, new_data: &[T]) {
        // TODO: make sure this buffer is big enough
        unsafe {
            upload(
                device,
                self.memory,
                std::mem::size_of_val(new_data) as _,
                new_data,
            )
        }
    }

    // TODO: clippy will whinge about this
    pub fn _is_empty(&self) -> bool {
        self.len == 0
    }
}

unsafe fn upload<T: Copy>(
    device: &ash::Device,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    data: &[T],
) {
    let ptr = device
        .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
        .unwrap();
    let mut slice = Align::new(ptr, align_of::<T>() as u64, size);
    slice.copy_from_slice(data);
    device.unmap_memory(memory);
}
