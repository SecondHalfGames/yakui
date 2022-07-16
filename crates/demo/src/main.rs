mod apps;
mod graphics;

use std::time::Instant;

use clap::Parser;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use apps::App;
use graphics::Graphics;
use yakui::paint::{Texture, TextureFormat};
use yakui::{TextureId, UVec2};

const MONKEY_PNG: &[u8] = include_bytes!("../assets/monkey.png");

#[derive(Parser)]
struct Args {
    app: App,
}

pub struct AppState {
    pub time: f32,
    pub monkey: TextureId,
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

    let force_scale: Option<f32> = std::env::var("YAKUI_FORCE_SCALE")
        .ok()
        .and_then(|s| s.parse().ok());

    if let Some(scale) = force_scale {
        yak_window.set_automatic_scale_factor(false);
        yak.set_scale_factor(scale);
    }

    let monkey = yak.add_texture(load_texture(MONKEY_PNG));
    let mut state = AppState { time: 0.0, monkey };

    let start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if yak_window.handle_event(&mut yak, &event) {
            return;
        }

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

                yak.start();
                app(&state);
                yak.finish();
                graphics.paint(&mut yak, &mut yak_renderer);

                profiling::finish_frame!();
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if button == winit::event::MouseButton::Left {
                    println!("Left mouse button {state:?}");
                }
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

fn init_logging() {
    env_logger::builder()
        .filter_module("wgpu_hal::auxil::dxgi", log::LevelFilter::Off)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .init();
}

fn main() {
    #[cfg(feature = "profile")]
    let _client = tracy_client::Client::start();

    init_logging();

    pollster::block_on(run());
}
