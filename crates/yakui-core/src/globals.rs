use anymap::AnyMap;

/// Struct to hold generic global states.
///
/// See: [`crate::dom::Dom`], [`crate::paint::PaintDom`]
#[derive(Debug)]
pub struct Globals {
    inner: AnyMap,
}

impl Globals {
    /// Initalizes a new global-state holder.
    pub fn new() -> Self {
        Self {
            inner: AnyMap::new(),
        }
    }

    /// Get a piece of global state mutably or initialize it with the given function.
    pub fn get_mut<T, F>(&mut self, init: F) -> &mut T
    where
        T: 'static,
        F: FnOnce() -> T,
    {
        self.inner.entry::<T>().or_insert_with(init)
    }

    /// Get a piece of global state and clone it or initialize it with the given function.
    pub fn get<T, F>(&mut self, init: F) -> T
    where
        T: 'static + Clone,
        F: FnOnce() -> T,
    {
        self.inner.entry::<T>().or_insert_with(init).clone()
    }
}
