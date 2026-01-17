use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use yakui::UVec2;

use crate::multisampling::Multisampling;

/// A helper for setting up rendering with winit and wgpu
pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    format: wgpu::TextureFormat,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    size: UVec2,
    sample_count: u32,
    multisampling: Multisampling,

    pub renderer: yakui_wgpu::YakuiWgpu,
}

impl Graphics {
    pub async fn new<T: HasDisplayHandle + HasWindowHandle>(
        window: &T,
        mut size: UVec2,
        sample_count: u32,
    ) -> Self {
        // FIXME: On web, we're receiving (0, 0) as the initial size of the
        // window, which makes wgpu upset. If we hit that case, let's just make
        // up a size and let it get fixed later.
        if size == UVec2::new(0, 0) {
            size = UVec2::new(800, 600);
        }

        let instance = wgpu::Instance::default();
        let surface = unsafe {
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(window)
                    .expect("Could not create wgpu surface from window"),
            )
        }
        .expect("Could not create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            view_formats: Vec::new(),
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // yakui_wgpu takes paint output from yakui and renders it for us using
        // wgpu.
        let renderer = yakui_wgpu::YakuiWgpu::new(&device, &queue);

        Self {
            device,
            queue,

            format,
            surface,
            surface_config,
            size,
            sample_count,
            multisampling: Multisampling::new(),

            renderer,
        }
    }

    pub fn resize(&mut self, new_size: UVec2) {
        if new_size.x > 0 && new_size.y > 0 && new_size != self.size {
            self.size = new_size;
            self.surface_config.width = new_size.x;
            self.surface_config.height = new_size.y;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    #[profiling::function]
    pub fn paint(&mut self, yak: &mut yakui::Yakui, bg: wgpu::Color) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let surface = self.multisampling.surface_info(
            &self.device,
            &view,
            self.size,
            self.format,
            self.sample_count,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface.color_attachment.view,
                    depth_slice: None,
                    resolve_target: surface.color_attachment.resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }

        let clear = encoder.finish();

        let paint_yak = self.renderer.paint(yak, &self.device, &self.queue, surface);

        self.queue.submit([clear, paint_yak]);
        output.present();
    }
}
