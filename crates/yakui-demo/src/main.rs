use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn ui() {
    yakui::vertical(|| {
        yakui::fsbox([40.0, 20.0]);
        yakui::fsbox([30.0, 30.0]);
        yakui::fsbox([60.0, 20.0]);
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut yak = yakui::State::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        yakui_winit::handle_event(&mut yak, &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                yak.start();
                ui();
                yak.finish();
            }

            _ => (),
        }
    });
}
