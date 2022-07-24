use std::mem::size_of;

use bytemuck::{bytes_of, NoUninit};

pub struct Buffer {
    len: usize,
    cpu_buffer: Vec<u8>,
    gpu_buffer: Option<wgpu::Buffer>,
    gpu_buffer_len: usize,
    usage: wgpu::BufferUsages,
}

impl Buffer {
    pub fn new(usage: wgpu::BufferUsages) -> Self {
        Self {
            len: 0,
            cpu_buffer: Vec::new(),
            gpu_buffer: None,
            gpu_buffer_len: 0,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cpu_buffer.clear();
    }

    #[allow(unused)]
    pub fn push(&mut self, value: &impl NoUninit) {
        self.len += 1;
        self.cpu_buffer.extend(bytes_of(value));
    }

    pub fn extend<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
        T: NoUninit,
    {
        profiling::scope!("Buffer::extend");

        let iter = iter.into_iter();
        self.len += iter.len();
        self.cpu_buffer.reserve(iter.len() * size_of::<T>());

        for v in iter {
            self.cpu_buffer.extend(bytes_of(&v));
        }
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> &wgpu::Buffer {
        let buffer = if self.gpu_buffer_len >= self.cpu_buffer.len() {
            self.gpu_buffer.as_ref().unwrap()
        } else {
            // Buffer needs to grow or be created
            let size = self.cpu_buffer.len().next_power_of_two();
            self.gpu_buffer_len = size;

            let desc = wgpu::BufferDescriptor {
                label: Some("yakui vertices"),
                size: size as u64,
                usage: self.usage,
                mapped_at_creation: false,
            };

            let buffer = device.create_buffer(&desc);
            self.gpu_buffer.insert(buffer)
        };

        queue.write_buffer(buffer, 0, &self.cpu_buffer);
        buffer
    }
}
