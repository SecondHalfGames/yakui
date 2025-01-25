use std::fmt;

use yakui_core::widget::Widget;
use yakui_core::{context, Response};

#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Scope<T> {
    value: T,
}

impl<T: 'static> Scope<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn show(self, children: impl FnOnce()) -> Response<ScopeResponse> {
        let dom = context::dom();
        let res = dom.begin_widget::<ScopeWidget>(());
        let scope_index = dom.dynamic_scope().push_scope();
        dom.get_mut(res.id).unwrap().dynamic_scope_index = Some(scope_index);

        dom.dynamic_scope().write_item(self.value);

        children();

        dom.dynamic_scope().pop_scope();
        dom.end_widget::<ScopeWidget>(res.id);

        res
    }
}

impl<T> fmt::Debug for Scope<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Scope")
    }
}

pub type ScopeResponse = ();

#[derive(Debug)]
pub struct ScopeWidget {}

impl Widget for ScopeWidget {
    type Props<'a> = ();
    type Response = ScopeResponse;

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}
}
