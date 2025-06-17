use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use winit::window::WindowId;
use winit::{event::WindowEvent, event_loop::EventLoop};
use yakui::{button, row, widgets::List, CrossAxisAlignment, Yakui};
use yakui_app::YakuiApp;

struct DemoApp;

impl App for DemoApp {
    fn render(&mut self) {
        row(|| {
            button("Not stretched");
            let mut col = List::column();
            col.cross_axis_alignment = CrossAxisAlignment::Stretch;
            col.show(|| {
                button("Button 1");
                button("Button 2");
                button("Button 3");
            });
        });
    }
}

trait App {
    fn render(&mut self);
}

struct AppHandler<T: App> {
    app: T,
    yak: Yakui,

    context: Option<YakuiApp>,

    // https://github.com/rust-windowing/winit/issues/3406
    #[cfg(target_os = "ios")]
    redraw_requested: bool,
}

impl<T: App> AppHandler<T> {
    fn new(app: T) -> Self {
        Self {
            app: app,
            yak: Yakui::new(),

            context: None,

            #[cfg(target_os = "ios")]
            redraw_requested: false,
        }
    }
}

impl<T: App> ApplicationHandler for AppHandler<T> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let window_attributes = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                WindowAttributes::default()
            }

            #[cfg(target_arch = "wasm32")]
            {
                use platform::web::WindowAttributesWeb;

                WindowAttributes::default().with_platform_attributes(Box::new(
                    WindowAttributesWeb::default().with_append(true), // Automatically append `Canvas` to DOM.
                ));
            }
        };

        let window = event_loop.create_window(window_attributes).unwrap();

        self.context = Some(YakuiApp::new(window, 4));
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.context = None;
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let context = match self.context.as_mut() {
            Some(context) => context,
            None => return,
        };

        if context.handle_window_event(&mut self.yak, &event, event_loop) {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                // Render
                if let Some(graphics) = context.graphics_mut() {
                    self.yak.start();
                    self.app.render();
                    self.yak.finish();

                    graphics.paint(&mut self.yak, wgpu::Color::BLACK);
                }

                // Request Redraw
                context.window.request_redraw();

                #[cfg(target_os = "ios")]
                {
                    self.redraw_requested = true;
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        #[cfg(target_os = "ios")]
        if self.redraw_requested {
            if let Some(context) = self.context.as_ref() {
                context.window.request_redraw();
            }
            self.redraw_requested = false;
        }
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        EventLoop::new()
            .unwrap()
            .run_app(AppHandler::new(DemoApp))
            .unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");

        EventLoop::new()
            .unwrap()
            .spawn_app(AppHandler::new(DemoApp))
            .unwrap();
    }
}
