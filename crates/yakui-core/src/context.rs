//! Provides access to the currently active DOM.

use std::cell::RefCell;
use std::thread::LocalKey;

use crate::dom::Dom;

type Storage<T> = RefCell<Option<T>>;

thread_local! {
    static CURRENT_DOM: Storage<Dom> = const { RefCell::new(None) };
}

/// If there is a DOM currently being updated on this thread, returns a handle
/// to it.
///
/// # Panics
/// Panics if there is no DOM currently being updated on this thread.
pub fn dom() -> Dom {
    CURRENT_DOM.with_borrow(|context| {
        context
            .as_ref()
            .unwrap_or_else(|| panic!("Cannot get access to a Dom when one is not in progress."))
            .clone()
    })
}

/// Binds a DOM on the current thread.
///
/// # Panics
/// Panics if there is already a DOM being updated on this thread.
pub fn bind_dom(dom: &Dom) {
    bind(&CURRENT_DOM, dom.clone());
}

/// Unbinds the DOM on the current thread.
///
/// # Panics
/// Panics if there is no DOM currently being updated on this thread.
pub fn unbind_dom() {
    unbind(&CURRENT_DOM);
}

fn bind<T>(storage: &'static LocalKey<Storage<T>>, value: T) {
    storage.with(move |current| {
        let mut current = current.borrow_mut();
        if current.is_some() {
            panic!("Cannot start a Dom while one is already in progress.");
        }
        *current = Some(value);
    });
}

fn unbind<T>(storage: &'static LocalKey<Storage<T>>) {
    storage.with(|current| {
        let mut current = current.borrow_mut();
        current.take().unwrap_or_else(|| {
            panic!("Cannot stop a Dom when there is not one in progress.");
        });
    });
}
