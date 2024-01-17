pub(crate) struct Samplers {
    nearest_nearest_nearest: wgpu::Sampler,
    linear_linear_nearest: wgpu::Sampler,
    nearest_linear_nearest: wgpu::Sampler,
    linear_nearest_nearest: wgpu::Sampler,
    nearest_nearest_linear: wgpu::Sampler,
    linear_linear_linear: wgpu::Sampler,
    nearest_linear_linear: wgpu::Sampler,
    linear_nearest_linear: wgpu::Sampler,
}

impl Samplers {
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::FilterMode::{Linear, Nearest};

        Self {
            nearest_nearest_nearest: sampler(device, Nearest, Nearest, Nearest),
            linear_linear_nearest: sampler(device, Linear, Linear, Nearest),
            nearest_linear_nearest: sampler(device, Nearest, Linear, Nearest),
            linear_nearest_nearest: sampler(device, Linear, Nearest, Nearest),
            nearest_nearest_linear: sampler(device, Nearest, Nearest, Linear),
            linear_linear_linear: sampler(device, Linear, Linear, Linear),
            nearest_linear_linear: sampler(device, Nearest, Linear, Linear),
            linear_nearest_linear: sampler(device, Linear, Nearest, Linear),
        }
    }

    pub fn get(
        &self,
        min: wgpu::FilterMode,
        max: wgpu::FilterMode,
        mipmap: wgpu::FilterMode,
    ) -> &wgpu::Sampler {
        use wgpu::FilterMode::{Linear, Nearest};

        match (min, max, mipmap) {
            (Nearest, Nearest, Nearest) => &self.nearest_nearest_nearest,
            (Linear, Linear, Nearest) => &self.linear_linear_nearest,
            (Nearest, Linear, Nearest) => &self.nearest_linear_nearest,
            (Linear, Nearest, Nearest) => &self.linear_nearest_nearest,
            (Nearest, Nearest, Linear) => &self.nearest_nearest_linear,
            (Linear, Linear, Linear) => &self.linear_linear_linear,
            (Nearest, Linear, Linear) => &self.nearest_linear_linear,
            (Linear, Nearest, Linear) => &self.linear_nearest_linear,
        }
    }
}

fn sampler(
    device: &wgpu::Device,
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter,
        min_filter,
        mipmap_filter,
        ..Default::default()
    })
}
