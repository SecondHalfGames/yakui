use crate::component::Component;
use crate::context::Context;
use crate::dom::{Dom, LayoutDom};
use crate::layout::{Constraints, Layout};
use crate::rect::Rect;
use crate::Event;

#[derive(Debug)]
pub struct State {
    dom: Dom,
    layout: LayoutDom,
    viewport: Rect,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let dom = Dom::new();
        let layout = LayoutDom::new();

        Self {
            dom,
            layout,
            viewport: Rect::ZERO,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SetViewport(viewport) => {
                self.viewport = viewport;
            }
        }
    }

    pub fn start(&mut self) {
        let context = Context::current();

        if let Some(mut snapshot) = self.dom.take_snapshot() {
            snapshot.clear();
            context.borrow_mut().start(snapshot);
        } else {
            panic!("Cannot call start() when already started.");
        }
    }

    pub fn finish(&mut self) {
        let context = Context::current();
        let mut context = context.borrow_mut();

        if let Some(snapshot) = context.take_snapshot() {
            self.dom.apply(snapshot);
        } else {
            panic!("Cannot call finish() when not started.");
        }

        let constraints = Constraints {
            min: None,
            max: Some(self.viewport.size()),
        };

        self.layout.calculate_all(&self.dom, constraints);
    }

    pub fn draw(&self) -> Layout {
        todo!()
    }
}
