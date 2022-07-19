use std::collections::VecDeque;
use std::fmt::Write as _;

use yakui_core::dom::Dom;
use yakui_core::geometry::{Rect, Vec2};
use yakui_core::layout::LayoutDom;

pub extern crate insta;
pub extern crate yakui_core;

#[macro_export]
macro_rules! run {
    ($body:expr) => {
        let test = Test::new();
        run!(test, $body);
    };

    ($test:expr, $body:expr) => {
        let test = $test;
        let mut state = ::yakui_test::yakui_core::State::new();
        state.set_unscaled_viewport(test.viewport);
        state.start();
        {
            $body
        }
        state.finish();

        let dom = state.dom();
        let layout = state.layout_dom();
        let view = ::yakui_test::view(dom, layout);

        ::yakui_test::insta::assert_snapshot!(view);
    };
}

pub struct Test {
    pub viewport: Rect,
}

impl Test {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            viewport: Rect::from_pos_size(Vec2::ZERO, Vec2::new(1000.0, 1000.0)),
        }
    }
}

pub fn view(dom: &Dom, layout: &LayoutDom) -> String {
    let mut output = String::new();
    let mut to_visit = VecDeque::new();

    let root = dom.get(dom.root()).unwrap();
    to_visit.extend(root.children.iter().rev().map(|&id| (id, 0)));

    while let Some((id, depth)) = to_visit.pop_back() {
        let node = dom.get(id).unwrap();
        let indent = "  ".repeat(depth);
        let name = trim_name(node.widget.type_name());

        let layout_info = match layout.get(id) {
            Some(info) => {
                let [x, y] = info.rect.pos().to_array();
                let [w, h] = info.rect.size().to_array();

                format!("pos({x}, {y}) size({w}, {h})")
            }
            None => "(no layout)".to_owned(),
        };

        writeln!(output, "{indent}- {name} {layout_info}").unwrap();

        to_visit.extend(node.children.iter().rev().map(|&id| (id, depth + 1)));
    }

    output
}

/// Trim modules off of the name of a Rust type name.
fn trim_name(name: &'static str) -> &'static str {
    name.rsplit("::").next().unwrap()
}
