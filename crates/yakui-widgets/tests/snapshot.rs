use std::collections::VecDeque;
use std::fmt::Write as _;

use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::Alignment;

#[test]
fn colored_box() {
    Test::new("colored_box").run(|| {
        yakui_widgets::colored_box(Color3::WHITE, [50.0, 50.0]);
    });
}

#[test]
fn column() {
    Test::new("column").run(|| {
        yakui_widgets::column(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row() {
    Test::new("row").run(|| {
        yakui_widgets::row(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn align_center() {
    Test::new("align_center").run(|| {
        yakui_widgets::center(|| {
            rect_50x50();
        });
    });
}

#[test]
fn align_bottom_right() {
    Test::new("align_bottom_right").run(|| {
        yakui_widgets::align(Alignment::BOTTOM_RIGHT, || {
            rect_50x50();
        });
    });
}

fn rect(w: f32, h: f32) {
    yakui_widgets::colored_box(Color3::WHITE, [w, h]);
}

fn rect_50x50() {
    rect(50.0, 50.0);
}

struct Test {
    name: &'static str,
    viewport: Rect,
}

impl Test {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            viewport: Rect::from_pos_size(Vec2::ZERO, Vec2::new(1000.0, 1000.0)),
        }
    }

    fn run<F: FnOnce()>(self, callback: F) {
        let mut state = yakui_core::State::new();
        state.set_unscaled_viewport(self.viewport);
        state.start();
        callback();
        state.finish();

        let dom = state.dom();
        let layout = state.layout_dom();

        insta::assert_snapshot!(self.name, view(dom, layout));
    }
}

fn view(dom: &Dom, layout: &LayoutDom) -> String {
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
