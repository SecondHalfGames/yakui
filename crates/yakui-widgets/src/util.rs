use yakui_core::context;
use yakui_core::widget::Widget;
use yakui_core::Response;

/// Show a widget with the given children and props.
pub fn widget_children<T, F>(children: F, props: T::Props<'_>) -> Response<T::Response>
where
    T: Widget,
    F: FnOnce(),
{
    let dom = context::dom();
    let response = dom.begin_widget::<T>(props);
    children();
    dom.end_widget::<T>(response.id);
    response
}

/// Show a widget with the given props.
pub fn widget<T>(props: T::Props<'_>) -> Response<T::Response>
where
    T: Widget,
{
    let dom = context::dom();
    dom.do_widget::<T>(props)
}
