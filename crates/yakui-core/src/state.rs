use crate::context;
use crate::dom::Dom;
use crate::event::{Event, EventResponse};
use crate::geometry::Rect;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::paint::PaintDom;

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

        let response = self.input.handle_event(&self.dom, &self.layout, &event);

        if let Event::ViewportChanged(viewport) = event {
            self.layout.set_unscaled_viewport(viewport);
        }

        context::unbind_dom();
        context::unbind_input();
        response == EventResponse::Sink
    }

    /// Set the size of the viewport in physical units.
    pub fn set_unscaled_viewport(&mut self, view: Rect) {
        self.layout.set_unscaled_viewport(view);
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
        self.paint.start();
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

    /// Returns access to the state's DOM.
    pub fn dom(&self) -> &Dom {
        &self.dom
    }

    /// Returns access to the state's Layout DOM.
    pub fn layout_dom(&self) -> &LayoutDom {
        &self.layout
    }
}
