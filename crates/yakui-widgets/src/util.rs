use yakui_core::{context::Context, Component};

pub fn component_children<T, F>(children: F, props: T::Props) -> T::Response
where
    T: Component,
    F: FnOnce(),
{
    let context = Context::active();

    let index = context.borrow_mut().dom_mut().begin_component::<T>(props);
    children();
    let res = context.borrow_mut().dom_mut().end_component::<T>(index);
    res
}

pub fn component<T>(props: T::Props) -> T::Response
where
    T: Component,
{
    let context = Context::active();

    let res = context.borrow_mut().dom_mut().do_component::<T>(props);
    res
}
