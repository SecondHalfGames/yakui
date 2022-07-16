mod examples;
mod graphics;

use std::time::Instant;

use clap::Parser;
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use examples::Example;
use graphics::Graphics;
use yakui::paint::{Texture, TextureFormat};
use yakui::{TextureId, UVec2};

const MONKEY_PNG: &[u8] = include_bytes!("../assets/monkey.png");

/// Run a yakui example.
#[derive(Parser)]
struct Args {
    example: Example,
}

/// This is the state that we provide to each demo.
///
/// It's not required to package your state into a struct, but this is a
/// convenient way for us to pass some common stuff to each example.
pub struct ExampleState {
    /// Some examples have basic animations or changing state, so they use the
    /// current time as an input.
    pub time: f32,

    /// `TextureId` is a handle to a texture we previously gave to yakui. This
    /// is an image that's usable from any of the examples.
    pub monkey: TextureId,
}

async fn run() {
    // The demo app uses clap to parse arguments. We have a little glue here to
    // also grab which example to run.
    let args = Args::parse();
    let example = args.example.function();

    // Normal winit setup for an EventLoop and Window.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(format!("yakui Demo: {:?}", args.example))
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    // The demo app has a small graphics abstraction using wgpu.
    let mut graphics = Graphics::new(&window).await;

    // Create our yakui state. This is where our UI will be built, laid out, and
    // calculations for painting will happen.
    let mut yak = yakui::State::new();

    // yakui_wgpu takes paint output from yakui and renders it for us using
    // wgpu.
    let mut yak_renderer =
        yakui_wgpu::State::new(&graphics.device, &graphics.queue, graphics.surface_format());

    // yakui_winit processes winit events and applies them to our yakui state.
    let mut yak_window = yakui_winit::State::new(&window);

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
        yak.set_scale_factor(scale);
    }

    // Preload a texture for the examples to use and set up some default state
    // that we'll modify later.
    let monkey = yak.add_texture(load_texture(MONKEY_PNG));
    let mut state = ExampleState { time: 0.0, monkey };

    let start = Instant::now();
    let mut is_init = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // yakui_winit will return whether it handled an event. This means that
        // yakui believes it should handle that event exclusively, like if a
        // button in the UI was clicked.
        if yak_window.handle_event(&mut yak, &event) {
            return;
        }

        match event {
            Event::MainEventsCleared => {
                state.time = (Instant::now() - start).as_secs_f32();

                // Every frame, call yak.start() to begin building the UI for
                // this frame. Any yakui widget calls that happen on this thread
                // between start() and finish() will be applied to this yakui
                // State.
                yak.start();

                // Here, we call out to our example code. See `src/examples` for
                // the code, which runs each frame.
                example(&state);

                // Finish building the UI and compute this frame's layout.
                yak.finish();

                // The example graphics abstraction calls yak.paint() to get
                // access to the underlying PaintDom, which holds all the state
                // about how to paint widgets.
                graphics.paint(&mut yak, &mut yak_renderer);

                profiling::finish_frame!();
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                // This print is a handy way to show which mouse events are
                // handled by yakui, and which ones will make it to the
                // underlying application.
                if button == winit::event::MouseButton::Left {
                    println!("Left mouse button {state:?}");
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::NewEvents(cause) => {
                if cause == StartCause::Init {
                    is_init = true;
                } else {
                    is_init = false;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Ignore any resize events that happen during Winit's
                // initialization in order to avoid racing the wgpu swapchain
                // and causing issues.
                //
                // https://github.com/rust-windowing/winit/issues/2094
                if is_init {
                    return;
                }

                graphics.resize(size);
            }

            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => {
                graphics.resize(*new_inner_size);
            }

            _ => (),
        }
    });
}

/// This function takes some bytes and turns it into a yakui `Texture` object so
/// that we can reference it later in our UI.
fn load_texture(bytes: &[u8]) -> Texture {
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let size = UVec2::new(image.width(), image.height());

    Texture::new(TextureFormat::Rgba8Srgb, size, image.into_raw())
}

/// Initialize our logging, adjusting the default log levels of some of our
/// noisier dependencies.
fn init_logging() {
    env_logger::builder()
        .filter_module("wgpu_hal::auxil::dxgi", log::LevelFilter::Off)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Enables the user to override the scaling of the demo app by setting an
/// environment variable.
fn get_scale_override() -> Option<f32> {
    std::env::var("YAKUI_FORCE_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
}

fn main() {
    #[cfg(feature = "profile")]
    let _client = tracy_client::Client::start();

    init_logging();

    pollster::block_on(run());
}
