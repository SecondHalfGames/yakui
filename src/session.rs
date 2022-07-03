use crate::component::Component;
use crate::context::Context;
use crate::dom::Dom;
use crate::registry::Registry;

#[derive(Debug)]
pub struct State {
    dom: Dom,
    registry: Registry,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let registry = Registry::new();
        let dom = Dom::new(registry.clone());

        Self { dom, registry }
    }

    pub fn register<T, P>(&self)
    where
        T: Component<P>,
        P: 'static,
    {
        self.registry.register::<T, P>();
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
