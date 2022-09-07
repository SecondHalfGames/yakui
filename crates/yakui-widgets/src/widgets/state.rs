use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util;

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

    pub fn show(self) -> Response<StateWidget<T>> {
        util::widget(self)
    }
}

impl<T> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("State")
    }
}

pub struct StateWidget<T> {
    value: Option<Rc<RefCell<T>>>,
}

impl<T: 'static> Widget for StateWidget<T> {
    type Props = State<T>;
    type Response = Rc<RefCell<T>>;

    fn new() -> Self {
        Self { value: None }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        let value = self
            .value
            .get_or_insert_with(|| Rc::new(RefCell::new((props.default)())))
            .clone();

        value
    }
}

impl<T> fmt::Debug for StateWidget<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("StateWidget")
    }
}
