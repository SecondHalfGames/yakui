use winit::{
    dpi::PhysicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};
use yakui_wgpu::SurfaceInfo;

/// A helper for setting up rendering with winit and wgpu
pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    format: wgpu::TextureFormat,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,

    window: yakui_winit::YakuiWinit,
    pub renderer: yakui_wgpu::YakuiWgpu,

    /// Tracks whether winit is still initializing
    is_init: bool,
}

impl Graphics {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface =
            unsafe { instance.create_surface(&window) }.expect("Could not create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            view_formats: Vec::new(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        // yakui_wgpu takes paint output from yakui and renders it for us using
        // wgpu.
        let renderer = yakui_wgpu::YakuiWgpu::new(&device, &queue);

        // yakui_winit processes winit events and applies them to our yakui
        // state.
        let window = yakui_winit::YakuiWinit::new(window);

        Self {
            device,
            queue,

            format,
            surface,
            surface_config,
            size,

            renderer,
            window,

            is_init: true,
        }
    }

    pub fn renderer_mut(&mut self) -> &mut yakui_wgpu::YakuiWgpu {
        &mut self.renderer
    }

    pub fn window_mut(&mut self) -> &mut yakui_winit::YakuiWinit {
        &mut self.window
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    #[cfg_attr(feature = "profiling", profiling::function)]
    pub fn paint(&mut self, yak: &mut yakui_core::Yakui, bg: wgpu::Color) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        let clear = encoder.finish();

        let surface = SurfaceInfo {
            format: self.format,
            sample_count: 1,
            color_attachment: &view,
            resolve_target: None,
        };
        let paint_yak = self.renderer.paint(yak, &self.device, &self.queue, surface);

        self.queue.submit([clear, paint_yak]);
        output.present();
    }

    pub fn handle_event<T>(
        &mut self,
        yak: &mut yakui::Yakui,
        event: &Event<T>,
        control_flow: &mut ControlFlow,
    ) -> bool {
        // yakui_winit will return whether it handled an event. This means that
        // yakui believes it should handle that event exclusively, like if a
        // button in the UI was clicked.
        if self.window.handle_event(yak, event) {
            return true;
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::NewEvents(cause) => {
                if *cause == StartCause::Init {
                    self.is_init = true;
                } else {
                    self.is_init = false;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Ignore any resize events that happen during Winit's
                // initialization in order to avoid racing the wgpu swapchain
                // and causing issues.
                //
                // https://github.com/rust-windowing/winit/issues/2094
                if self.is_init {
                    return false;
                }

                self.resize(*size);
            }

            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => {
                self.resize(**new_inner_size);
            }

            _ => (),
        }

        false
    }
}
