use yakui_core::context;
use yakui_core::widget::Widget;
use yakui_core::Response;

pub fn widget_children<T, F>(children: F, props: T::Props) -> Response<T>
where
    T: Widget,
    F: FnOnce(),
{
    let dom = context::dom();
    let index = dom.begin_widget::<T>(props);
    children();
    dom.end_widget::<T>(index)
}

pub fn widget<T>(props: T::Props) -> Response<T>
where
    T: Widget,
{
    let dom = context::dom();
    dom.do_widget::<T>(props)
}
