use std::collections::HashMap;

pub(crate) struct Samplers {
    samplers: HashMap<SamplerKey, wgpu::Sampler>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SamplerKey {
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::MipmapFilterMode,
    address_mode: wgpu::AddressMode,
}

impl Samplers {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut samplers = HashMap::new();

        let filter_modes = [wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest];
        let mipmap_filter_modes = [
            wgpu::MipmapFilterMode::Linear,
            wgpu::MipmapFilterMode::Nearest,
        ];
        let address_modes = [wgpu::AddressMode::ClampToEdge, wgpu::AddressMode::Repeat];
        for min_filter in filter_modes {
            for mag_filter in filter_modes {
                for mipmap_filter in mipmap_filter_modes {
                    for address_mode in address_modes {
                        let key = SamplerKey {
                            min_filter,
                            mag_filter,
                            mipmap_filter,
                            address_mode,
                        };
                        samplers.insert(
                            key,
                            sampler(device, min_filter, mag_filter, mipmap_filter, address_mode),
                        );
                    }
                }
            }
        }

        Self { samplers }
    }

    pub fn get(
        &self,
        min: wgpu::FilterMode,
        max: wgpu::FilterMode,
        mipmap: wgpu::MipmapFilterMode,
        address_mode: wgpu::AddressMode,
    ) -> &wgpu::Sampler {
        self.samplers
            .get(&SamplerKey {
                min_filter: min,
                mag_filter: max,
                mipmap_filter: mipmap,
                address_mode,
            })
            .unwrap()
    }
}

fn sampler(
    device: &wgpu::Device,
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::MipmapFilterMode,
    address_mode: wgpu::AddressMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter,
        min_filter,
        mipmap_filter,
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        ..Default::default()
    })
}
