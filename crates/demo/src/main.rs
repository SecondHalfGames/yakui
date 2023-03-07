use winit::{
    event::Event,
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
    let mut graphics = Graphics::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if graphics.handle_event(&mut yak, &event, control_flow) {
            return;
        }

        match event {
            Event::MainEventsCleared => {
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::dpi::LogicalSize;
                    use winit::event::WindowEvent;
                    use winit::window::WindowId;

                    let web_window = web_sys::window().unwrap();

                    let event: Event<'_, ()> = Event::WindowEvent {
                        window_id: unsafe { WindowId::dummy() },
                        event: WindowEvent::Resized(window.inner_size()),
                    };

                    let width = web_window.inner_width().unwrap().as_f64().unwrap();
                    let height = web_window.inner_height().unwrap().as_f64().unwrap();
                    window.set_inner_size(LogicalSize::new(width, height));
                    graphics.handle_event(&mut yak, &event, control_flow);
                }

                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                yak.start();
                app();
                yak.finish();

                graphics.paint(&mut yak, wgpu::Color::BLACK);
            }

            _ => (),
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
