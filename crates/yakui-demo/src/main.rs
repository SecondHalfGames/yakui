mod graphics;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use graphics::Graphics;

fn ui() {
    yakui::vertical(|| {
        yakui::fsbox([40.0, 20.0]);
        yakui::fsbox([30.0, 30.0]);
        yakui::fsbox([60.0, 20.0]);
    });
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics = Graphics::new(&window).await;

    let mut yak = yakui::State::new();
    let mut yak_renderer = yakui_wgpu::State::new(&graphics.device, graphics.surface_format());

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

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                graphics.resize(size);
            }

            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => {
                graphics.resize(*new_inner_size);
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                yak.start();
                ui();
                yak.finish();

                graphics.draw(&yak, &mut yak_renderer);
            }

            _ => (),
        }
    });
}

fn main() {
    pollster::block_on(run());
}
