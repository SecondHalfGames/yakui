use crate::component::Component;
use crate::context::Context;
use crate::dom::{Dom, DomLayout};
use crate::layout::Layout;
use crate::registry::Registry;
use crate::Constraints;

#[derive(Debug)]
pub struct State {
    dom: Dom,
    layout: DomLayout,
    registry: Registry,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let registry = Registry::new();
        let dom = Dom::new(registry.clone());
        let layout = DomLayout::new();

        Self {
            dom,
            layout,
            registry,
        }
    }

    pub fn register<T>(&self)
    where
        T: Component,
    {
        self.registry.register::<T>();
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
    }

    pub fn layout(&mut self, constraints: Constraints) {
        self.layout.calculate_all(&self.dom, constraints);
    }

    pub fn draw(&self) -> Layout {
        todo!()
    }
}
