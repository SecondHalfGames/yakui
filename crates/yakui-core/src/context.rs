//! Provides access to the currently active DOM while it is being updated.

use std::cell::{Ref, RefCell};

use crate::dom::Dom;

type Storage = RefCell<Option<Dom>>;

thread_local! {
    static CURRENT_DOM: Storage = RefCell::new(None);
}

/// If there is a DOM currently being updated on this thread, returns a
/// reference to it.
///
/// # Panics
/// Panics if there is no DOM currently being updated on this thread.
pub fn dom() -> Ref<'static, Dom> {
    CURRENT_DOM.with(|context| {
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

pub(crate) fn bind_dom(dom: &Dom) {
    CURRENT_DOM.with(|current| {
        let mut current = current.borrow_mut();
        if current.is_some() {
            panic!("Cannot start a Dom while one is already in progress.");
        }
        *current = Some(dom.clone());
    });
}

pub(crate) fn unbind_dom() {
    CURRENT_DOM.with(|dom| {
        let mut dom = dom.borrow_mut();

        dom.take().unwrap_or_else(|| {
            panic!("Cannot stop a Dom when there is not one in progress.");
        });
    });
}

unsafe fn extend_lifetime<'a, T>(value: &'a T) -> &'static T {
    std::mem::transmute::<&'a T, &'static T>(value)
}
