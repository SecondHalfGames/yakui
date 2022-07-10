use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::dom::Dom;

type Storage = Rc<RefCell<Option<Dom>>>;

thread_local! {
    static CURRENT_CONTEXT: Storage = Rc::new(RefCell::new(None));
}

pub fn dom() -> Ref<'static, Dom> {
    CURRENT_CONTEXT.with(|context| {
        // SAFETY: Rust won't give us a 'static reference here because this
        // thread could send the 'static value to another thread and then die
        // prematurely. This would be bad.
        //
        // However, we have some good news! Ref<T> is not Send and never will
        // be. We can rest assured that this reference will never outlive the
        // thread it originated from.
        let context = unsafe { extend_lifetime(context) };

        let context = context.borrow();
        Ref::map(context, |context| {
            context.as_ref().unwrap_or_else(|| {
                panic!("Cannot get access to a Dom when one is not in progress.")
            })
        })
    })
}

pub(crate) fn give_dom(dom: Dom) {
    let context = CURRENT_CONTEXT.with(Rc::clone);
    let mut context = context.borrow_mut();
    if context.is_some() {
        panic!("Cannot start a Dom while one is already in progress.");
    }
    *context = Some(dom);
}

pub(crate) fn take_dom() -> Dom {
    let context = CURRENT_CONTEXT.with(Rc::clone);
    let mut context = context.borrow_mut();

    context.take().unwrap_or_else(|| {
        panic!("Cannot stop a Dom when there is not one in progress.");
    })
}

unsafe fn extend_lifetime<'a, T>(value: &'a T) -> &'static T {
    std::mem::transmute::<&'a T, &'static T>(value)
}
