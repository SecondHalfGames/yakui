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

/// See also: https://github.com/rust-lang/rust/issues/154024
#[macro_export]
macro_rules! builtin_auto_builder {
    ( $name:ident: $type:ty ) => {
        pub fn $name(self, $name: $type) -> Self {
            Self { $name, ..self }
        }
    };
}

#[macro_export]
macro_rules! auto_builder {
    ( $name:ident: f32 ) => { $crate::builtin_auto_builder!($name: f32); };
    ( $name:ident: f64 ) => { $crate::builtin_auto_builder!($name: f64); };
    ( $name:ident: u8    ) => { $crate::builtin_auto_builder!($name: u8   ); };
    ( $name:ident: u16   ) => { $crate::builtin_auto_builder!($name: u16  ); };
    ( $name:ident: u32   ) => { $crate::builtin_auto_builder!($name: u32  ); };
    ( $name:ident: u64   ) => { $crate::builtin_auto_builder!($name: u64  ); };
    ( $name:ident: u128  ) => { $crate::builtin_auto_builder!($name: u128 ); };
    ( $name:ident: usize ) => { $crate::builtin_auto_builder!($name: usize); };
    ( $name:ident: i8    ) => { $crate::builtin_auto_builder!($name: i8   ); };
    ( $name:ident: i16   ) => { $crate::builtin_auto_builder!($name: i16  ); };
    ( $name:ident: i32   ) => { $crate::builtin_auto_builder!($name: i32  ); };
    ( $name:ident: i64   ) => { $crate::builtin_auto_builder!($name: i64  ); };
    ( $name:ident: i128  ) => { $crate::builtin_auto_builder!($name: i128 ); };
    ( $name:ident: isize ) => { $crate::builtin_auto_builder!($name: isize); };

    ( $name:ident: $type:ty ) => {
        pub fn $name<T: Into<$type>>(self, $name: T) -> Self {
            Self {
                $name: $name.into(),
                ..self
            }
        }
    };
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
                $crate::util::paste::paste! {
                    $crate::auto_builder!($name: $type);
                }
            )*
        }
    };
}

pub use paste;

pub use auto_builders;
