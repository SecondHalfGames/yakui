use yakui_core::{context::Context, Widget};

pub fn widget_children<T, F>(children: F, props: T::Props) -> T::Response
where
    T: Widget,
    F: FnOnce(),
{
    let context = Context::active();

    let index = context.borrow_mut().dom_mut().begin_widget::<T>(props);
    children();
    let res = context.borrow_mut().dom_mut().end_widget::<T>(index);
    res
}

pub fn widget<T>(props: T::Props) -> T::Response
where
    T: Widget,
{
    let context = Context::active();

    let res = context.borrow_mut().dom_mut().do_widget::<T>(props);
    res
}
