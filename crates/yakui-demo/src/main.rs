mod graphics;

use std::time::Instant;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use graphics::Graphics;
use yakui::Color3;

fn ui(time: f32) {
    yakui::vertical(|| {
        yakui::horizontal(|| {
            let x = 50.0 * time.sin();
            let y = 20.0 * (time + 1.0).sin();

            yakui::fsbox([100.0 + x, 100.0 + y], Color3::RED);
            yakui::fsbox([40.0, 30.0], Color3::GREEN);
            yakui::fsbox([60.0, 40.0], Color3::BLUE);
        });
        yakui::fsbox([200.0, 100.0], Color3::REBECCA_PURPLE);
    });
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics = Graphics::new(&window).await;

    let mut yak = yakui::State::new();
    let mut yak_renderer = yakui_wgpu::State::new(&graphics.device, graphics.surface_format());
    let mut yak_window = yakui_winit::State::new();

    let start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        yak_window.handle_event(&mut yak, &event);

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
                let time = (Instant::now() - start).as_secs_f32();
                ui(time);
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
