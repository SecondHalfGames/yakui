use crate::dom::Dom;
use crate::input::{InputState, NavDirection};
use crate::layout::LayoutDom;
use crate::widget::NavigateContext;
use crate::WidgetId;

pub fn navigate(
    dom: &Dom,
    layout: &LayoutDom,
    input: &InputState,
    dir: NavDirection,
) -> Option<WidgetId> {
    let mut current = input.selection();

    while let Some(id) = current {
        let node = dom.get(id).unwrap();
        let ctx = NavigateContext { dom, layout, input };

        if let Some(new_id) = ctx.try_navigate(id, dir) {
            return Some(new_id);
        }

        current = node.parent;
    }

    None
}
