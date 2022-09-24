pub struct PipelineCache {
    layout: wgpu::PipelineLayout,
    current: Option<Cached>,
}

struct Cached {
    pipeline: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
    samples: u32,
}

impl PipelineCache {
    pub fn new(layout: wgpu::PipelineLayout) -> Self {
        Self {
            layout,
            current: None,
        }
    }

    pub fn get<'a, F>(
        &'a mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        samples: u32,
        init: F,
    ) -> &'a wgpu::RenderPipeline
    where
        F: FnOnce(
            &wgpu::Device,
            &wgpu::PipelineLayout,
            wgpu::TextureFormat,
            u32,
        ) -> wgpu::RenderPipeline,
    {
        match &mut self.current {
            Some(existing) if existing.format == format && existing.samples == samples => (),
            _ => {
                let pipeline = init(device, &self.layout, format, samples);

                self.current = Some(Cached {
                    pipeline,
                    format,
                    samples,
                });
            }
        }

        &self.current.as_ref().unwrap().pipeline
    }
}
