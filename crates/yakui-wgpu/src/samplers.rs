pub(crate) struct Samplers {
    nearest: wgpu::Sampler,
    linear: wgpu::Sampler,
    nearest_linear: wgpu::Sampler,
    linear_nearest: wgpu::Sampler,
}

impl Samplers {
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::FilterMode::{Linear, Nearest};

        Self {
            nearest: sampler(device, Nearest, Nearest),
            linear: sampler(device, Linear, Linear),
            nearest_linear: sampler(device, Nearest, Linear),
            linear_nearest: sampler(device, Linear, Nearest),
        }
    }

    pub fn get(&self, min: wgpu::FilterMode, max: wgpu::FilterMode) -> &wgpu::Sampler {
        use wgpu::FilterMode::{Linear, Nearest};

        match (min, max) {
            (Nearest, Nearest) => &self.nearest,
            (Linear, Linear) => &self.linear,
            (Nearest, Linear) => &self.nearest_linear,
            (Linear, Nearest) => &self.linear_nearest,
        }
    }
}

fn sampler(
    device: &wgpu::Device,
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter,
        min_filter,
        ..Default::default()
    })
}
