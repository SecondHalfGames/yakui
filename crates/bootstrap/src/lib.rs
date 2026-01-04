mod common;
mod custom_texture;
mod graphics;
mod multisampling;
mod with_sdl3;
mod with_winit;

use std::fmt::Write;
use std::sync::Arc;

use yakui::cosmic_text::fontdb;
use yakui::font::Fonts;
use yakui::paint::TextureFilter;

pub use crate::common::*;

/// Bootstrap and start a new app, using the given function as the body of the
/// function, which runs every frame.
pub fn start(body: impl ExampleBody) {
    #[cfg(feature = "tracy-client")]
    let _client = tracy_client::Client::start();

    init_logging();

    run(body);
}

fn run(body: impl ExampleBody) {
    let mut title = "yakui demo".to_owned();

    if let Some(scale) = get_scale_override() {
        write!(title, " (scale override {scale})").unwrap();
    }

    // Create our yakui state. This is where our UI will be built, laid out, and
    // calculations for painting will happen.
    let mut yak = yakui::Yakui::new();

    // Preload some textures for the examples to use.
    let monkey = yak.add_texture(load_texture(MONKEY_PNG, TextureFilter::Linear));
    let monkey_transparent = yak.add_texture({
        let mut texture = load_texture(MONKEY_PNG, TextureFilter::Linear);
        for pixel in texture.data_mut().chunks_exact_mut(4) {
            pixel[3] = 64;
        }
        texture
    });
    let monkey_blurred = yak.add_texture(load_texture(MONKEY_BLURRED_PNG, TextureFilter::Linear));
    let brown_inlay = yak.add_texture(load_texture(BROWN_INLAY_PNG, TextureFilter::Nearest));

    // Add a custom font for some of the examples.
    let fonts = yak.dom().get_global_or_init(Fonts::default);

    static HACK_REGULAR: &[u8] = include_bytes!("../assets/Hack-Regular.ttf");

    fonts.load_font_source(fontdb::Source::Binary(Arc::from(&HACK_REGULAR)));
    fonts.set_monospace_family("Hack");

    let state = ExampleState {
        time: 0.0,
        monkey,
        monkey_transparent,
        monkey_blurred,
        brown_inlay,
        custom: None,
    };

    let backend = get_backend();
    println!("Using windowing library {backend:?}");

    match backend {
        BootstrapBackend::Winit => {
            crate::with_winit::run(yak, state, title, body);
        }

        BootstrapBackend::Sdl3 => {
            crate::with_sdl3::run(yak, state, title, body);
        }
    }
}
