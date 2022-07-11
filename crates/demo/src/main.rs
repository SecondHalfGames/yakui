mod apps;
mod graphics;

use std::time::Instant;

use clap::Parser;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use apps::App;
use graphics::Graphics;
use yakui::{
    paint::{Texture, TextureFormat},
    UVec2,
};

const MONKEY_PNG: &[u8] = include_bytes!("../assets/monkey.png");

#[derive(Parser)]
struct Args {
    app: App,
}

pub struct AppState {
    pub time: f32,
    pub monkey: yakui::Index,
}

async fn run() {
    let args = Args::parse();
    let app = args.app.function();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics = Graphics::new(&window).await;

    let mut yak = yakui::State::new();
    let mut yak_renderer =
        yakui_wgpu::State::new(&graphics.device, &graphics.queue, graphics.surface_format());
    let mut yak_window = yakui_winit::State::new(&window);
    // yak_window.set_automatic_scale_factor(false);

    let monkey = yak.create_texture(load_texture(MONKEY_PNG));
    let mut state = AppState { time: 0.0, monkey };

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
                state.time = (Instant::now() - start).as_secs_f32();

                {
                    profiling::scope!("UI");

                    {
                        profiling::scope!("UI Create+Update");
                        yak.start();
                        app(&state);
                    }

                    {
                        profiling::scope!("UI Layout and Input");
                        yak.finish();
                    }
                }

                {
                    profiling::scope!("Rendering");
                    graphics.paint(&mut yak, &mut yak_renderer);
                }

                profiling::finish_frame!();
            }

            _ => (),
        }
    });
}

fn load_texture(bytes: &[u8]) -> Texture {
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let size = UVec2::new(image.width(), image.height());

    Texture::new(TextureFormat::Rgba8Srgb, size, image.into_raw())
}

fn main() {
    #[cfg(feature = "profile")]
    let _client = tracy_client::Client::start();

    env_logger::init();

    pollster::block_on(run());
}
