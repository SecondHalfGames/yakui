use crate::context::Context;
use crate::dom::{Dom, LayoutDom};
use crate::draw::Output;
use crate::Event;

#[derive(Debug)]
pub struct State {
    dom: Option<Dom>,
    layout: LayoutDom,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let dom = Some(Dom::new());
        let layout = LayoutDom::new();

        Self { dom, layout }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SetViewport(viewport) => {
                self.layout.viewport = viewport;
            }
        }
    }

    pub fn start(&mut self) {
        let context = Context::current();

        if let Some(mut dom) = self.dom.take() {
            dom.start();
            context.borrow_mut().start(dom);
        } else {
            panic!("Cannot call start() when already started.");
        }
    }

    pub fn finish(&mut self) {
        let context = Context::current();
        let mut context = context.borrow_mut();

        if let Some(dom) = context.take_dom() {
            self.dom = Some(dom);
        } else {
            panic!("Cannot call finish() when not started.");
        }

        self.layout.calculate_all(self.dom.as_ref().unwrap());
    }

    pub fn draw(&self) -> Output {
        Output::draw(self.dom.as_ref().unwrap(), &self.layout)
    }
}
