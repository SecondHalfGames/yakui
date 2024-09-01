use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util;

#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct State<T> {
    default: Box<dyn FnOnce() -> T>,
}

impl<T: 'static> State<T> {
    pub fn new<F>(default: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        Self {
            default: Box::new(default),
        }
    }

    pub fn show(self) -> Response<StateResponse<T>> {
        util::widget::<StateWidget<T>>(self)
    }
}

impl<T> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("State")
    }
}

pub struct StateResponse<T> {
    value: Rc<RefCell<T>>,
}

impl<T> StateResponse<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.value.borrow_mut()
    }

    pub fn set(&self, value: T) {
        self.value.replace(value);
    }
}

impl<T: Copy> StateResponse<T> {
    pub fn get(&self) -> T {
        *self.value.borrow()
    }

    pub fn modify<F>(&self, update: F)
    where
        F: FnOnce(T) -> T,
    {
        let mut handle = self.value.borrow_mut();
        *handle = update(*handle);
    }
}

pub struct StateWidget<T> {
    value: Option<Rc<RefCell<T>>>,
}

impl<T: 'static> Widget for StateWidget<T> {
    type Props<'a> = State<T>;
    type Response = StateResponse<T>;

    fn new() -> Self {
        Self { value: None }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        let value = self
            .value
            .get_or_insert_with(|| Rc::new(RefCell::new((props.default)())))
            .clone();

        StateResponse { value }
    }
}

impl<T> fmt::Debug for StateWidget<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("StateWidget")
    }
}
