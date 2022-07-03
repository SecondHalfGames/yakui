use crate::context::Context;
use crate::dom::Dom;

pub struct State {
    dom: Dom,
}

impl State {
    pub fn new() -> Self {
        Self { dom: Dom::new() }
    }

    pub fn start(&mut self) {
        let context = Context::current();

        if let Some(snapshot) = self.dom.take_snapshot() {
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
    }
}
