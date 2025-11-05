use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use yakui::{Rect, UVec2, Vec2, Yakui};
use yakui_winit::YakuiWinit;

use crate::{
    custom_texture, get_inset_override, get_sample_count, get_scale_override, graphics::Graphics,
    ExampleBody, ExampleState,
};

pub fn run<T: ExampleBody>(yak: Yakui, state: ExampleState, title: String, body: T) {
    // Normal winit setup for an EventLoop and Window.
    let event_loop = EventLoop::new().unwrap();
    let window_attribute = Window::default_attributes().with_title(title);

    // Set up some default state that we'll modify later.
    let mut app = App {
        yak,
        attributes: window_attribute,
        start: Instant::now(),
        state,
        window: None,
        graphics: None,
        yak_window: None,
        body,
    };

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app).unwrap();
}

struct App<T: ExampleBody> {
    state: ExampleState,
    yak: Yakui,

    attributes: WindowAttributes,
    start: Instant,

    window: Option<Window>,
    graphics: Option<Graphics>,
    yak_window: Option<YakuiWinit>,

    body: T,
}

impl<T: ExampleBody> ApplicationHandler for App<T> {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(self.attributes.clone()).unwrap();
        window.set_ime_allowed(true);

        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height);
        let sample_count = get_sample_count();

        let mut graphics = pollster::block_on(Graphics::new(&window, size, sample_count));
        let mut yak_window = yakui_winit::YakuiWinit::new(&window);

        // By default, yakui_winit will measure the system's scale factor and pass
        // it to yakui.
        //
        // Sometimes, it might be desirable to scale the UI by a different factor,
        // like if your game has a "UI scale" option, if you're writing tests, or
        // you want to ensure your widgets work at a different scale.
        //
        // In these examples, setting the YAKUI_FORCE_SCALE environment variable to
        // a number will override the automatic scaling.
        if let Some(scale) = get_scale_override() {
            yak_window.set_automatic_scale_factor(false);
            self.yak.set_scale_factor(scale);
        }

        // In these examples, set YAKUI_INSET to force the UI to be contained within
        // a sub-viewport with the given edge inset on all sides.
        let inset = get_inset_override();
        if inset.is_some() {
            yak_window.set_automatic_viewport(false);
        }

        let custom = graphics.renderer.add_texture(
            custom_texture::generate(&graphics.device, &graphics.queue),
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            wgpu::MipmapFilterMode::Nearest,
            wgpu::AddressMode::ClampToEdge,
        );
        self.state.custom = Some(custom);

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
        if self.yak_window.as_mut().unwrap().handle_window_event(
            &mut self.yak,
            &event,
            self.window.as_ref().unwrap(),
        ) {
            return;
        }

        // Handle window event.
        match event {
            WindowEvent::RedrawRequested => {
                self.state.time = (Instant::now() - self.start).as_secs_f32();

                {
                    profiling::scope!("Build UI");

                    // Every frame, call yak.start() to begin building the UI for
                    // this frame. Any yakui widget calls that happen on this thread
                    // between start() and finish() will be applied to this yakui
                    // State.
                    self.yak.start();

                    // Call out to the body of the program, passing in a bit of
                    // shared state that all the examples can use.
                    self.body.run(&mut self.state);

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

            WindowEvent::Resized(size) => {
                let inset = get_inset_override();
                if let Some(inset) = inset {
                    let size = Vec2::new(size.width as f32, size.height as f32);
                    self.yak.set_unscaled_viewport(Rect::from_pos_size(
                        Vec2::splat(inset),
                        size - Vec2::splat(inset * 2.0),
                    ));
                }

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
