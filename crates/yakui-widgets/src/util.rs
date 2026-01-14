use std::rc::Rc;

use yakui_core::context;
use yakui_core::widget::Widget;
use yakui_core::Response;

/// Show a widget with the given children and props.
#[track_caller]
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
#[track_caller]
pub fn widget<T>(props: T::Props<'_>) -> Response<T::Response>
where
    T: Widget,
{
    let dom = context::dom();
    dom.do_widget::<T>(props)
}

pub fn read_scope<T: 'static>() -> Option<Rc<T>> {
    let dom = context::dom();
    let current = dom.get_current().dynamic_scope_index?;
    dom.dynamic_scope().get(current)
}

#[macro_export]
macro_rules! auto_builders {
    (
        $struct:ident {
            $( $name:ident: $type:ty ),*
            $(,)?
        }
    ) => {
        impl $struct {
            $(
                pub fn $name<T: Into<$type>>(self, $name: T) -> Self {
                    Self { $name: $name.into(), ..self }
                }
            )*
        }
    };
}

pub use auto_builders;
