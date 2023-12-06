use winit::dpi::PhysicalSize;
use yakui_wgpu::SurfaceInfo;

pub(crate) struct Multisampling {
    sample_count: u32,
    format: wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    ms_view: Option<wgpu::TextureView>,
}

impl Multisampling {
    pub fn new() -> Self {
        Self {
            sample_count: 4,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            size: PhysicalSize::new(800, 600),
            ms_view: None,
        }
    }

    pub fn surface_info<'a>(
        &'a mut self,
        device: &wgpu::Device,
        view: &'a wgpu::TextureView,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> SurfaceInfo<'a> {
        if size != self.size || format != self.format || sample_count != self.sample_count {
            self.ms_view = None;
        }

        if sample_count == 1 {
            return SurfaceInfo {
                format,
                sample_count,
                color_attachment: view,
                resolve_target: None,
            };
        }

        let ms_view = self.ms_view.get_or_insert_with(|| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisampled Target"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                mip_level_count: 1,
                view_formats: &[],
            });

            texture.create_view(&Default::default())
        });

        SurfaceInfo {
            format,
            sample_count,
            color_attachment: ms_view,
            resolve_target: Some(view),
        }
    }
}
