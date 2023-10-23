mod custom_texture;

use std::fmt::Write;
use std::time::Instant;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use yakui::font::{Font, FontSettings, Fonts};
use yakui::paint::{Texture, TextureFilter, TextureFormat};
use yakui::{ManagedTextureId, Rect, TextureId, UVec2, Vec2};

const MONKEY_PNG: &[u8] = include_bytes!("../assets/monkey.png");
const MONKEY_BLURRED_PNG: &[u8] = include_bytes!("../assets/monkey-blurred.png");
const BROWN_INLAY_PNG: &[u8] = include_bytes!("../assets/brown_inlay.png");

/// This is the state that we provide to each demo.
///
/// It's not required to package your state into a struct, but this is a
/// convenient way for us to pass some common stuff to each example.
pub struct ExampleState {
    /// Some examples have basic animations or changing state, so they use the
    /// current time as an input.
    pub time: f32,

    /// `ManagedTextureId` is a texture owned by yakui. You can create one by
    /// giving yakui some image data; it'll be uploaded by the renderer.
    pub monkey: ManagedTextureId,
    pub monkey_blurred: ManagedTextureId,
    pub brown_inlay: ManagedTextureId,

    /// `TextureId` represents either a managed texture or a texture owned by
    /// the renderer. This image is generated in `custom_texture.rs` and
    /// uploaded with wgpu directly.
    pub custom: TextureId,
}

pub trait ExampleBody: 'static {
    fn run(&self, state: &mut ExampleState);
}

impl ExampleBody for fn() {
    fn run(&self, _state: &mut ExampleState) {
        (self)();
    }
}

impl ExampleBody for fn(&mut ExampleState) {
    fn run(&self, state: &mut ExampleState) {
        (self)(state);
    }
}

/// Boostrap and start a new app, using the given function as the body of the
/// function, which runs every frame.
pub fn start(body: impl ExampleBody) {
    #[cfg(feature = "profile")]
    let _client = tracy_client::Client::start();

    init_logging();

    pollster::block_on(run(body));
}

async fn run(body: impl ExampleBody) {
    let mut title = "yakui demo".to_owned();

    if let Some(scale) = get_scale_override() {
        write!(title, " (scale override {scale})").unwrap();
    }

    // Normal winit setup for an EventLoop and Window.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    // yakui_app has a helper for setting up winit and wgpu.
    let mut app = yakui_app::Graphics::new(&window).await;

    // Create our yakui state. This is where our UI will be built, laid out, and
    // calculations for painting will happen.
    let mut yak = yakui::Yakui::new();

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
        app.window_mut().set_automatic_scale_factor(false);
        yak.set_scale_factor(scale);
    }

    // In these examples, set YAKUI_INSET to force the UI to be contained within
    // a sub-viewport with the given edge inset on all sides.
    let inset = get_inset_override();
    if inset.is_some() {
        app.window_mut().set_automatic_viewport(false);
    }

    // Preload some textures for the examples to use.
    let monkey = yak.add_texture(load_texture(MONKEY_PNG, TextureFilter::Linear));
    let monkey_blurred = yak.add_texture(load_texture(MONKEY_BLURRED_PNG, TextureFilter::Linear));
    let brown_inlay = yak.add_texture(load_texture(BROWN_INLAY_PNG, TextureFilter::Nearest));
    let custom = app.renderer.add_texture(
        custom_texture::generate(&app.device, &app.queue),
        wgpu::FilterMode::Nearest,
        wgpu::FilterMode::Nearest,
    );

    // Add a custom font for some of the examples.
    let fonts = yak.dom().get_global_or_init(Fonts::default);
    let font = Font::from_bytes(
        include_bytes!("../assets/Hack-Regular.ttf").as_slice(),
        FontSettings::default(),
    )
    .unwrap();
    fonts.add(font, Some("monospace"));

    // Set up some default state that we'll modify later.
    let mut state = ExampleState {
        time: 0.0,
        monkey,
        monkey_blurred,
        brown_inlay,
        custom,
    };

    let start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if app.handle_event(&mut yak, &event, control_flow) {
            return;
        }

        match event {
            Event::MainEventsCleared => {
                state.time = (Instant::now() - start).as_secs_f32();

                {
                    profiling::scope!("Build UI");

                    // Every frame, call yak.start() to begin building the UI for
                    // this frame. Any yakui widget calls that happen on this thread
                    // between start() and finish() will be applied to this yakui
                    // State.
                    yak.start();

                    // Call out to the body of the program, passing in a bit of
                    // shared state that all the examples can use.
                    body.run(&mut state);

                    // Finish building the UI and compute this frame's layout.
                    yak.finish();
                }

                // The example graphics abstraction calls yak.paint() to get
                // access to the underlying PaintDom, which holds all the state
                // about how to paint widgets.
                app.paint(&mut yak, {
                    let bg = yakui::colors::BACKGROUND_1.to_linear();
                    wgpu::Color {
                        r: bg.x.into(),
                        g: bg.y.into(),
                        b: bg.z.into(),
                        a: 1.0,
                    }
                });

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
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Some(inset) = inset {
                    let size = Vec2::new(size.width as f32, size.height as f32);
                    yak.set_unscaled_viewport(Rect::from_pos_size(
                        Vec2::splat(inset),
                        size - Vec2::splat(inset * 2.0),
                    ));
                }
            }

            _ => (),
        }
    });
}

/// This function takes some bytes and turns it into a yakui `Texture` object so
/// that we can reference it later in our UI.
fn load_texture(bytes: &[u8], filter: TextureFilter) -> Texture {
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let size = UVec2::new(image.width(), image.height());

    let mut texture = Texture::new(TextureFormat::Rgba8Srgb, size, image.into_raw());
    texture.mag_filter = filter;
    texture
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

/// Enables the user to set a sub-viewport that the example should render into.
fn get_inset_override() -> Option<f32> {
    std::env::var("YAKUI_INSET")
        .ok()
        .and_then(|s| s.parse().ok())
}
