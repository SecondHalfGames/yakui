use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use yakui::{button, row, widgets::List, CrossAxisAlignment, Yakui};
use yakui_app::Graphics;

fn app() {
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

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut yak = Yakui::new();
    let mut graphics = Graphics::new(&window, 4).await;

    event_loop.set_control_flow(ControlFlow::Poll);
    #[allow(deprecated)] // winit!! :shake fist:
    event_loop
        .run(move |event, event_loop| match event {
            Event::AboutToWait => {
                window.request_redraw();
            }

            Event::NewEvents(cause) => {
                graphics.is_init = cause == StartCause::Init;
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                yak.start();
                app();
                yak.finish();

                graphics.paint(&mut yak, wgpu::Color::BLACK);
            }

            Event::WindowEvent { event, .. } => {
                graphics.handle_window_event(&mut yak, &event, event_loop, &window);
            }
            _ => (),
        })
        .unwrap();
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    #[allow(deprecated)]
    let window = event_loop
        .create_window(winit::window::Window::default_attributes())
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas().unwrap()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
