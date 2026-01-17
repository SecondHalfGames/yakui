use std::time::Instant;

use glam::{UVec2, Vec2};
use thunderdome::{Arena, Index};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use yakui::paint::{PaintCall, UserPaintCallId};
use yakui::util::widget;
use yakui::widget::Widget;
use yakui::{Color, Yakui};
use yakui_wgpu::{DrawCall, SurfaceInfo, Vertex};
use yakui_winit::YakuiWinit;

#[derive(Debug)]
struct MyCustomRenderer {
    objects: Arena<MyCustomRenderedObject>,
    initial_time: Instant,
}

impl MyCustomRenderer {
    pub fn add(&mut self, object: MyCustomRenderedObject) -> UserPaintCallId {
        self.objects.insert(object).to_bits()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn new() -> Self {
        Self {
            objects: Arena::new(),
            initial_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MyCustomRenderedObject {
    color: Color,
}

impl MyCustomRenderedObject {
    pub fn show(self) -> yakui::Response<()> {
        widget::<MyCustomRenderedWidget>(self)
    }
}

#[derive(Debug)]
struct MyCustomRenderedWidget {
    props: MyCustomRenderedObject,
}

impl Widget for MyCustomRenderedWidget {
    type Props<'a> = MyCustomRenderedObject;

    type Response = ();

    fn new() -> Self {
        Self {
            props: MyCustomRenderedObject {
                color: Color::CLEAR,
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props
    }

    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        _constraints: yakui::Constraints,
    ) -> yakui::Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        yakui::Vec2::new(80., 80.)
    }

    fn paint(&self, ctx: yakui::widget::PaintContext<'_>) {
        let id = ctx
            .paint
            .globals
            .get_mut(MyCustomRenderer::new)
            .add(self.props);

        ctx.paint.add_user_call(id);
    }
}

fn my_custom_paint(
    yakui_wgpu: &mut yakui_wgpu::YakuiWgpu,
    state: &mut yakui_core::Yakui,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface: SurfaceInfo<'_>,
) {
    // --- yakui ---
    yakui_wgpu.set_paint_limits(state);
    let paint = state.paint();

    yakui_wgpu.update_textures(device, paint, queue);

    // If there's nothing to paint, well... don't paint!
    let layers = &paint.layers;
    if layers.iter().all(|layer| layer.calls.is_empty()) {
        return;
    }

    // If the surface has a size of zero, well... don't paint either!
    if paint.surface_size().x == 0.0 || paint.surface_size().y == 0.0 {
        return;
    }

    yakui_wgpu.vertices.clear();
    yakui_wgpu.indices.clear();
    yakui_wgpu.texture_bindgroup_cache.clear();

    let mut draw_calls = Vec::with_capacity(layers.len());
    // --- yakui ---

    let renderer = paint.globals.get_mut(MyCustomRenderer::new);

    let mut custom_draws = Arena::new();
    let elapsed = Instant::now()
        .duration_since(renderer.initial_time)
        .as_secs_f32();

    for (clip, call) in layers.iter().flat_map(|layer| &layer.calls) {
        match call {
            PaintCall::Internal(call) => {
                draw_calls.push(yakui_wgpu.build_draw_call(device, *clip, call));
            }
            PaintCall::User(id) => {
                let min = clip.pos();
                let max = clip.max();

                let x1y1 = min;
                let x1y2 = Vec2::new(min.x, max.y);
                let x2y1 = Vec2::new(max.x, min.y);
                let x2y2 = max;

                let index = Index::from_bits(*id).unwrap();
                let object = renderer.objects.get(index).unwrap();
                let mut color = object.color.to_linear();

                color.x = (color.x + elapsed).sin().abs();
                color.y = (color.y + elapsed).cos().abs();

                let my_indices = [0, 1, 2, 1, 3, 2];

                let vertices = [x1y1, x1y2, x2y1, x2y2].into_iter().map(|pos| Vertex {
                    position: paint.info.transform_vertex(pos, false),
                    texcoord: Vec2::default(),
                    color,
                });

                let base = yakui_wgpu.vertices.len() as u32;
                let indices = my_indices.iter().map(|&index| base + index as u32);

                let start = yakui_wgpu.indices.len() as u32;
                let end = start + indices.len() as u32;

                yakui_wgpu.vertices.extend(vertices);
                yakui_wgpu.indices.extend(indices);

                draw_calls.push((*clip, DrawCall::User(*id)));
                custom_draws.insert_at(index, start..end);
            }
        }
    }

    renderer.clear();

    // --- yakui ---
    let vertices = yakui_wgpu.vertices.upload(device, queue);
    let indices = yakui_wgpu.indices.upload(device, queue);
    // --- yakui ---

    {
        // --- yakui ---
        let main_pipeline =
            yakui_wgpu::main_pipeline(&mut yakui_wgpu.main_pipeline, device, &surface);
        let text_pipeline =
            yakui_wgpu::text_pipeline(&mut yakui_wgpu.text_pipeline, device, &surface);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("yakui Render Pass"),
            color_attachments: &[Some(surface.color_attachment)],
            ..Default::default()
        });

        render_pass.set_vertex_buffer(0, vertices.slice(..));
        render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

        let surface = paint.surface_size().as_uvec2();
        render_pass.set_viewport(0.0, 0.0, surface.x as f32, surface.y as f32, 0.0, 1.0);

        let mut last_clip = None;
        // --- yakui ---

        for (clip, draw_call) in draw_calls {
            // --- yakui ---
            if Some(clip) != last_clip {
                last_clip = Some(clip);

                let surface = paint.surface_size().as_uvec2();

                let pos = clip.pos().as_uvec2();
                let size = clip.size().as_uvec2();

                let max = (pos + size).min(surface);
                let size = UVec2::new(max.x.saturating_sub(pos.x), max.y.saturating_sub(pos.y));

                // If the scissor rect isn't valid, we can skip this
                // entire draw call.
                if pos.x > surface.x || pos.y > surface.y || size.x == 0 || size.y == 0 {
                    continue;
                }

                render_pass.set_scissor_rect(pos.x, pos.y, size.x, size.y);
            }
            // --- yakui ---

            match draw_call {
                DrawCall::Yakui(call) => {
                    yakui_wgpu::YakuiWgpu::draw_yakui(
                        &yakui_wgpu.texture_bindgroup_cache,
                        &mut render_pass,
                        main_pipeline,
                        text_pipeline,
                        call,
                    );
                }
                DrawCall::User(id) => {
                    let index_range = custom_draws.remove(Index::from_bits(id).unwrap()).unwrap();

                    render_pass.set_bind_group(0, &yakui_wgpu.texture_bindgroup_cache.default, &[]);
                    render_pass.draw_indexed(index_range, 0, 0..1);
                }
            }
        }
    }
}

/// A helper for setting up rendering with winit and wgpu
pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    format: wgpu::TextureFormat,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    size: UVec2,

    pub renderer: yakui_wgpu::YakuiWgpu,
}

impl Graphics {
    pub async fn new<T: HasDisplayHandle + HasWindowHandle>(window: &T, mut size: UVec2) -> Self {
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

        let surface = SurfaceInfo {
            format: self.format,
            sample_count: 1,
            color_attachment: wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            },
        };

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

        let paint_yak = {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("yakui Encoder"),
                });

            my_custom_paint(
                &mut self.renderer,
                yak,
                &self.device,
                &self.queue,
                &mut encoder,
                surface,
            );

            encoder.finish()
        };

        self.queue.submit([clear, paint_yak]);
        output.present();
    }
}

struct App {
    yak: Yakui,
    window: Option<Window>,
    graphics: Option<Graphics>,
    yak_window: Option<YakuiWinit>,
}

impl ApplicationHandler for App {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        window.set_ime_allowed(true);

        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height);

        let graphics = pollster::block_on(Graphics::new(&window, size));
        let yak_window = yakui_winit::YakuiWinit::new(&window);

        self.window = Some(window);
        self.graphics = Some(graphics);
        self.yak_window = Some(yak_window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .yak_window
            .as_mut()
            .unwrap()
            .handle_window_event(&mut self.yak, &event)
        {
            return;
        }

        // Handle window event.
        match event {
            WindowEvent::RedrawRequested => {
                {
                    profiling::scope!("Build UI");

                    // Every frame, call yak.start() to begin building the UI for
                    // this frame. Any yakui widget calls that happen on this thread
                    // between start() and finish() will be applied to this yakui
                    // State.
                    self.yak.start();

                    gui();

                    // Finish building the UI and compute this frame's layout.
                    self.yak.finish();
                }

                // The example graphics abstraction calls yak.paint() to get
                // access to the underlying PaintDom, which holds all the state
                // about how to paint widgets.
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics.paint(&mut self.yak, {
                        let bg = yakui::colors::BACKGROUND_1.to_linear();
                        wgpu::Color {
                            r: bg.x.into(),
                            g: bg.y.into(),
                            b: bg.z.into(),
                            a: 1.0,
                        }
                    });
                }

                profiling::finish_frame!();

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::MouseInput { state, button, .. } => {
                // This print is a handy way to show which mouse events are
                // handled by yakui, and which ones will make it to the
                // underlying application.
                if button == winit::event::MouseButton::Left {
                    println!("Left mouse button {state:?}");
                }
            }

            WindowEvent::Resized(..) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.resize(self.yak.surface_size().as_uvec2());
                }
            }

            WindowEvent::CloseRequested => {
                self.graphics = None;

                event_loop.exit();
            }

            _ => (),
        }
    }
}

/// Simple test to make sure wgpu backend renders properly with a custom renderer.
fn main() {
    // Normal winit setup for an EventLoop and Window.
    let event_loop = EventLoop::new().unwrap();

    // Set up some default state that we'll modify later.
    let mut app = App {
        yak: Yakui::new(),
        window: None,
        graphics: None,
        yak_window: None,
    };

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app).unwrap();
}

fn gui() {
    use yakui::{column, label, row, widgets::Text, Color};
    column(|| {
        row(|| {
            label("Hello, world!");

            let mut text = Text::new(48.0, "colored text!");
            text.style.color = Color::RED;
            text.show();
        });

        row(|| {
            MyCustomRenderedObject { color: Color::RED }.show();
            MyCustomRenderedObject {
                color: Color::GREEN,
            }
            .show();
            MyCustomRenderedObject { color: Color::BLUE }.show();
            MyCustomRenderedObject {
                color: Color::YELLOW,
            }
            .show();
            MyCustomRenderedObject {
                color: Color::WHITE,
            }
            .show();
            MyCustomRenderedObject {
                color: Color::FUCHSIA,
            }
            .show();
        });
    });
}
