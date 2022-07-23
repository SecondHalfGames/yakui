use yakui::{CrossAxisAlignment, MainAxisSize, Vec2};
use yakui_core::geometry::Color3;
use yakui_core::Alignment;
use yakui_test::{run, Test};
use yakui_widgets::widgets::{List, Pad, UnconstrainedBox};
use yakui_widgets::{
    align, center, colored_box, colored_box_container, column, expanded, pad, row,
};

#[test]
fn colored_box_basic() {
    run!({
        colored_box(Color3::RED, [50.0, 50.0]);
    });
}

#[test]
fn column_basic() {
    run!({
        column(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row_basic() {
    run!({
        row(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row_grow_first() {
    run!({
        row(|| {
            expanded(|| {
                colored_box(Color3::RED, [50.0, 50.0]);
            });
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_middle() {
    run!({
        row(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color3::GREEN, [50.0, 50.0]);
            });
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_last() {
    run!({
        row(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color3::BLUE, [50.0, 50.0]);
            });
        });
    });
}

#[test]
fn column_grow_middle() {
    run!({
        column(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color3::GREEN, [50.0, 50.0]);
            });
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn column_min() {
    run!({
        let mut column = List::column();
        column.main_axis_size = MainAxisSize::Min;
        column.show(|| {
            rect_50x50();
            rect(100, 50);
            rect(50, 100);
        });
    });
}

#[test]
fn column_cross_stretch() {
    run!({
        let mut column = List::column();
        column.cross_axis_alignment = CrossAxisAlignment::Stretch;
        column.show(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_start() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Start;
        row.show(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_center() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Center;
        row.show(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_end() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::End;
        row.show(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_stretch() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Stretch;
        row.show(|| {
            colored_box(Color3::RED, [50.0, 50.0]);
            colored_box(Color3::GREEN, [50.0, 50.0]);
            colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

/// When given infinite constraints, widgets like List need to pick the minimum
/// size that fits their content, not infinity.
#[test]
fn row_unconstrained() {
    run!({
        let container = UnconstrainedBox::new();
        container.show(|| {
            row(|| {
                colored_box(Color3::RED, [50.0, 50.0]);
                colored_box(Color3::GREEN, [50.0, 50.0]);
                colored_box(Color3::BLUE, [50.0, 50.0]);
            });
        });
    });
}

#[test]
fn align_center() {
    run!({
        center(|| {
            rect_50x50();
        });
    });
}

#[test]
fn align_bottom_right() {
    run!({
        align(Alignment::BOTTOM_RIGHT, || {
            rect_50x50();
        });
    });
}

#[test]
fn pad_basic() {
    let padding = Pad::all(20.0);

    run!({
        align(Alignment::TOP_LEFT, || {
            colored_box_container(Color3::RED, || {
                pad(padding, || {
                    colored_box(Color3::GREEN, Vec2::splat(50.0));
                });
            });
        });
    });
}

#[test]
fn pad_empty() {
    let padding = Pad::all(20.0);

    run!({
        align(Alignment::TOP_LEFT, || {
            colored_box_container(Color3::RED, || {
                pad(padding, || {});
            });
        });
    });
}

fn rect<V: Dim>(w: V, h: V) {
    colored_box(Color3::WHITE, [w.to_f32(), h.to_f32()]);
}

fn rect_50x50() {
    rect(50.0, 50.0);
}

trait Dim {
    fn to_f32(self) -> f32;
}

impl Dim for f32 {
    fn to_f32(self) -> f32 {
        self
    }
}

impl Dim for i32 {
    fn to_f32(self) -> f32 {
        self as f32
    }
}
