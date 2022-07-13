use thunderdome::Index;

use crate::dom::Dom;
use crate::event::Event;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::paint::{PaintDom, Texture};
use crate::{context, EventResponse};

#[derive(Debug)]
pub struct State {
    dom: Option<Dom>,
    layout: LayoutDom,
    paint: PaintDom,
    input: InputState,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            dom: Some(Dom::new()),
            layout: LayoutDom::new(),
            paint: PaintDom::new(),
            input: InputState::new(),
        }
    }

    /// Handle the given event. Returns `true` if the event should by sunk by
    /// yakui.
    pub fn handle_event(&mut self, event: Event) -> bool {
        log::debug!("State::handle_event({event:?})");

        let dom = match &self.dom {
            Some(dom) => dom,
            None => panic!("Cannot handle_event() while DOM is being built."),
        };

        match event {
            Event::SetViewport(viewport) => {
                self.layout.set_unscaled_viewport(viewport);
                false
            }
            Event::MoveMouse(pos) => {
                self.input.mouse_moved(dom, &self.layout, pos);
                false
            }
            Event::MouseButtonChanged(button, down) => {
                let response = self
                    .input
                    .mouse_button_changed(dom, &self.layout, button, down);

                response == EventResponse::Sink
            }
        }
    }

    pub fn create_texture(&mut self, texture: Texture) -> Index {
        self.paint.create_texture(texture)
    }

    pub fn start(&mut self) {
        if let Some(dom) = self.dom.take() {
            dom.start();
            context::give_dom(dom);
        } else {
            panic!("Cannot call start() when already started.");
        }
    }

    pub fn finish(&mut self) {
        let dom = self.dom.insert(context::take_dom());
        dom.finish();

        self.layout.calculate_all(dom);
        self.input.finish();
    }

    pub fn paint(&mut self) -> &PaintDom {
        let dom = self.dom.as_ref().unwrap_or_else(|| {
            panic!("Cannot paint() while DOM is being built.");
        });

        self.paint.paint_all(dom, &self.layout);
        &self.paint
    }

    pub fn textures(&self) -> impl Iterator<Item = (Index, &Texture)> {
        self.paint.textures()
    }

    pub fn set_scale_factor(&mut self, factor: f32) {
        self.layout.set_scale_factor(factor);
    }
}
