use core::cell::Cell;
use std::sync::Arc;

use yakui::cosmic_text::fontdb;
use yakui::font::Fonts;
use yakui::paint::{Texture, TextureFilter, TextureFormat};
use yakui::util::widget;
use yakui::widget::Widget;
use yakui::{ManagedTextureId, TextureId, UVec2, Vec2};

pub const OPENMOJI: &[u8] = include_bytes!("../assets/OpenMoji-color-glyf_colr_0.ttf");

pub const MONKEY_PNG: &[u8] = include_bytes!("../assets/monkey.png");
pub const MONKEY_BLURRED_PNG: &[u8] = include_bytes!("../assets/monkey-blurred.png");
pub const BROWN_INLAY_PNG: &[u8] = include_bytes!("../assets/brown_inlay.png");

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
    pub monkey_transparent: ManagedTextureId,
    pub monkey_blurred: ManagedTextureId,
    pub brown_inlay: ManagedTextureId,

    /// `TextureId` represents either a managed texture or a texture owned by
    /// the renderer. This image is generated in `custom_texture.rs` and
    /// uploaded with wgpu directly.
    pub custom: Option<TextureId>,
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

/// This function takes some bytes and turns it into a yakui `Texture` object so
/// that we can reference it later in our UI.
pub fn load_texture(bytes: &[u8], filter: TextureFilter) -> Texture {
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let size = UVec2::new(image.width(), image.height());

    let mut texture = Texture::new(TextureFormat::Rgba8Srgb, size, image.into_raw());
    texture.mag_filter = filter;
    texture
}

/// Initialize our logging, adjusting the default log levels of some of our
/// noisier dependencies.
pub fn init_logging() {
    env_logger::builder()
        .filter_module("wgpu_hal::auxil::dxgi", log::LevelFilter::Off)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Enables the user to override the scaling of the demo app by setting an
/// environment variable.
pub fn get_scale_override() -> Option<f32> {
    std::env::var("YAKUI_FORCE_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
}

/// Enables the user to set a sub-viewport that the example should render into.
pub fn get_inset_override() -> Option<f32> {
    std::env::var("YAKUI_INSET")
        .ok()
        .and_then(|s| s.parse().ok())
}

/// Enables the user to override the number of multisampling samples that yakui
/// uses, defaulting to 4x MSAA.
pub fn get_sample_count() -> u32 {
    std::env::var("YAKUI_SAMPLE_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
}

#[derive(Debug)]
pub enum BootstrapBackend {
    Winit,
    Sdl3,
}

pub fn get_backend() -> BootstrapBackend {
    std::env::var("YAKUI_WINDOW_LIB")
        .ok()
        .map(|s| match s.as_str() {
            "winit" => BootstrapBackend::Winit,
            "sdl3" => BootstrapBackend::Sdl3,
            _ => panic!(
                "Unknown window library '{s}'; valid window libraries are 'winit' and 'sdl3'"
            ),
        })
        .unwrap_or(BootstrapBackend::Winit)
}

#[derive(Debug)]
struct LoadCommonFontsWidget {
    loaded: Cell<bool>,
}

impl Widget for LoadCommonFontsWidget {
    type Props<'a> = ();

    type Response = ();

    fn new() -> Self {
        Self {
            loaded: Cell::default(),
        }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}

    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        _constraints: yakui::Constraints,
    ) -> yakui::Vec2 {
        if !self.loaded.get() {
            let fonts = ctx.dom.get_global_or_init(Fonts::default);

            fonts.load_system_fonts();
            fonts.load_font_source(fontdb::Source::Binary(Arc::from(&OPENMOJI)));

            self.loaded.set(true);
        }

        Vec2::ZERO
    }
}

pub fn load_common_fonts() {
    widget::<LoadCommonFontsWidget>(());
}
