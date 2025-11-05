use crate::context;
use crate::dom::Dom;
use crate::event::{Event, EventResponse};
use crate::geometry::{Rect, Vec2};
use crate::id::ManagedTextureId;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::paint::{PaintDom, PaintLimits, Texture};

/// The entrypoint for yakui.
#[derive(Debug)]
pub struct Yakui {
    dom: Dom,
    layout: LayoutDom,
    paint: PaintDom,
    input: InputState,
}

impl Yakui {
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

        if let Event::ViewportChanged(viewport) = &event {
            self.layout.set_unscaled_viewport(*viewport);
        }

        let response = self.input.handle_event(&self.dom, &self.layout, event);

        context::unbind_dom();
        response == EventResponse::Sink
    }

    /// Creates a texture for use within yakui.
    pub fn add_texture(&mut self, texture: Texture) -> ManagedTextureId {
        self.paint.add_texture(texture)
    }

    /// Returns an iterator of all textures managed by yakui.
    pub fn textures(&self) -> impl Iterator<Item = (ManagedTextureId, &Texture)> {
        self.paint.textures()
    }

    /// Set the size of the surface the yakui is being rendered onto.
    pub fn set_surface_size(&mut self, size: Vec2) {
        self.paint.set_surface_size(size);
    }

    /// Return the current size of the primary surface.
    pub fn surface_size(&self) -> Vec2 {
        self.paint.surface_size()
    }

    /// Set the size and position of the viewport in physical units.
    pub fn set_unscaled_viewport(&mut self, view: Rect) {
        self.layout.set_unscaled_viewport(view);
        self.paint.set_unscaled_viewport(view);
    }

    /// Retrieve the scale factor currently used by yakui.
    pub fn scale_factor(&self) -> f32 {
        self.layout.scale_factor()
    }

    /// Manually sets the scale factor used for laying out widgets.
    ///
    /// Platform integrations will usually do this automatically. If you'd like
    /// to override that value, like to enable the user to set their own UI
    /// scale, this is the method to use.
    pub fn set_scale_factor(&mut self, factor: f32) {
        self.layout.set_scale_factor(factor);
        self.paint.set_scale_factor(factor);
    }

    /// Starts building the DOM on this thread.
    ///
    /// Once this method is called, widgets can be created on this thread and
    /// they will automatically be linked to this State.
    ///
    /// When finished, call [`Dom::finish`].
    pub fn start(&mut self) {
        self.dom.start();
        self.input.start(&self.dom, &self.layout);
        self.paint.start();

        context::bind_dom(&self.dom);
    }

    /// Finishes building the DOM. Must be called on a thread that previously
    /// called [`Dom::start`].
    ///
    /// This method will finalize the DOM for this frame and compute layouts.
    pub fn finish(&mut self) {
        context::unbind_dom();

        self.dom.finish(&self.input);
        self.layout.sync_removals(&self.dom.removed_nodes());
        self.layout
            .calculate_all(&self.dom, &self.input, &self.paint);
        self.input.finish(&self.dom, &self.layout);
    }

    /// Calculates the geometry needed to render the current state and gives
    /// access to the [`PaintDom`], which holds information about how to paint
    /// widgets.
    pub fn paint(&mut self) -> &PaintDom {
        self.paint.paint_all(&self.dom, &self.layout);
        &self.paint
    }

    /// Returns access to the state's DOM.
    pub fn dom(&self) -> &Dom {
        &self.dom
    }

    /// Returns access to the state's Layout DOM.
    pub fn layout_dom(&self) -> &LayoutDom {
        &self.layout
    }

    /// Returns access to the state's Paint DOM.
    pub fn paint_dom(&self) -> &PaintDom {
        &self.paint
    }

    /// Sets the paint limits, should be called once by rendering backends.
    pub fn set_paint_limit(&mut self, limits: PaintLimits) {
        self.paint.set_limit(limits)
    }

    /// Tells whether a widget is currently looking for text input, like a
    /// focused textbox.
    pub fn text_input_enabled(&self) -> bool {
        self.input.text_input_enabled()
    }

    /// Gets the text cursor, if any.
    pub fn get_text_cursor(&self) -> Option<Rect> {
        self.input.get_text_cursor()
    }
}
