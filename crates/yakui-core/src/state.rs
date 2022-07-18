use crate::context;
use crate::dom::Dom;
use crate::event::{Event, EventResponse};
use crate::id::TextureId;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::paint::{PaintDom, Texture};

/// The entrypoint for yakui.
#[derive(Debug)]
pub struct State {
    dom: Dom,
    layout: LayoutDom,
    paint: PaintDom,
    input: InputState,
}

impl State {
    /// Creates a new yakui State.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            dom: Dom::new(),
            layout: LayoutDom::new(),
            paint: PaintDom::new(),
            input: InputState::new(),
        }
    }

    /// Handles the given event. Returns `true` if the event was sunk by yakui
    /// and should not be processed by the application.
    pub fn handle_event(&mut self, event: Event) -> bool {
        log::debug!("State::handle_event({event:?})");

        context::bind_dom(&self.dom);
        context::bind_input(&self.input);

        let sink = match event {
            Event::ViewportChanged(viewport) => {
                self.layout.set_unscaled_viewport(viewport);
                false
            }
            Event::CursorMoved(pos) => {
                self.input.mouse_moved(&self.dom, &self.layout, pos);
                false
            }
            Event::MouseButtonChanged(button, down) => {
                let response =
                    self.input
                        .mouse_button_changed(&self.dom, &self.layout, button, down);

                response == EventResponse::Sink
            }
            Event::KeyChanged(key, down) => {
                let response = self
                    .input
                    .keyboard_key_changed(&self.dom, &self.layout, key, down);

                response == EventResponse::Sink
            }
            Event::TextInput(_c) => {
                // TODO
                false
            }
        };

        context::unbind_dom();
        context::unbind_input();
        sink
    }

    /// Creates a texture for use within yakui.
    pub fn add_texture(&mut self, texture: Texture) -> TextureId {
        self.paint.add_texture(texture)
    }

    /// Returns an iterator of all textures managed by yakui.
    pub fn textures(&self) -> impl Iterator<Item = (TextureId, &Texture)> {
        self.paint.textures()
    }

    /// Manually sets the scale factor used for laying out widgets.
    ///
    /// Platform integrations will usually do this automatically. If you'd like
    /// to override that value, like to enable the user to set their own UI
    /// scale, this is the method to use.
    pub fn set_scale_factor(&mut self, factor: f32) {
        self.layout.set_scale_factor(factor);
    }

    /// Starts building the DOM on this thread.
    ///
    /// Once this method is called, widgets can be created on this thread and
    /// they will automatically be linked to this State.
    ///
    /// When finished, call [`Dom::finish`].
    pub fn start(&mut self) {
        self.dom.start();
        context::bind_dom(&self.dom);
        context::bind_input(&self.input);
    }

    /// Finishes building the DOM. Must be called on a thread that previously
    /// called [`Dom::start`].
    ///
    /// This method will finalize the DOM for this frame and compute layouts.
    pub fn finish(&mut self) {
        context::unbind_dom();
        context::unbind_input();

        self.dom.finish();
        self.layout.calculate_all(&self.dom);
        self.input.finish();
    }

    /// Calculates the geometry needed to render the current state and gives
    /// access to the [`PaintDom`], which holds information about how to paint
    /// widgets.
    pub fn paint(&mut self) -> &PaintDom {
        self.paint.set_viewport(self.layout.viewport());
        self.paint.paint_all(&self.dom, &self.layout);
        &self.paint
    }
}
