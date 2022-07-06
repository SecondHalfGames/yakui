use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::Dom;

thread_local! {
    static CURRENT_CONTEXT: Rc<RefCell<Context>> = Rc::new(RefCell::new(Context::new()));
}

pub struct Context {
    dom: Option<Dom>,
}

impl Context {
    const fn new() -> Self {
        Self { dom: None }
    }

    pub fn dom_mut(&mut self) -> &mut Dom {
        self.dom.as_mut().unwrap()
    }

    pub(crate) fn start(&mut self, dom: Dom) {
        self.dom = Some(dom);
    }

    pub(crate) fn take_dom(&mut self) -> Option<Dom> {
        self.dom.take()
    }

    pub fn current() -> Rc<RefCell<Self>> {
        CURRENT_CONTEXT.with(Rc::clone)
    }

    pub fn active() -> Rc<RefCell<Self>> {
        let context = Self::current();

        if context.borrow().dom.is_none() {
            panic!("cannot call this method without an active yakui Session");
        }

        context
    }
}
