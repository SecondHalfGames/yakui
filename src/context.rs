use std::cell::RefCell;
use std::rc::Rc;

use crate::snapshot::Snapshot;

thread_local! {
    static CURRENT_CONTEXT: Rc<RefCell<Context>> = Rc::new(RefCell::new(Context::new()));
}

pub(crate) struct Context {
    snapshot: Option<Snapshot>,
}

impl Context {
    const fn new() -> Self {
        Self { snapshot: None }
    }

    pub(crate) fn snapshot_mut(&mut self) -> &mut Snapshot {
        self.snapshot.as_mut().unwrap()
    }

    pub(crate) fn start(&mut self, mut snapshot: Snapshot) {
        snapshot.clear();
        self.snapshot = Some(snapshot);
    }

    pub(crate) fn take_snapshot(&mut self) -> Option<Snapshot> {
        self.snapshot.take()
    }

    pub(crate) fn current() -> Rc<RefCell<Self>> {
        CURRENT_CONTEXT.with(Rc::clone)
    }

    pub(crate) fn active() -> Rc<RefCell<Self>> {
        let context = Self::current();

        if context.borrow().snapshot.is_none() {
            panic!("cannot call this method without an active yakui Session");
        }

        context
    }
}
