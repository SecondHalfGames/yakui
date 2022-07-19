use yakui::{CrossAxisAlignment, MainAxisSize};
use yakui_core::geometry::Color3;
use yakui_core::Alignment;
use yakui_test::{run, Test};
use yakui_widgets::widgets::{List, UnconstrainedBox};

#[test]
fn colored_box() {
    run!({
        yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
    });
}

#[test]
fn column() {
    run!({
        yakui_widgets::column(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row() {
    run!({
        yakui_widgets::row(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row_grow_first() {
    run!({
        yakui_widgets::row(|| {
            yakui_widgets::expanded(|| {
                yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            });
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_middle() {
    run!({
        yakui_widgets::row(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::expanded(|| {
                yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            });
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_last() {
    run!({
        yakui_widgets::row(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::expanded(|| {
                yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
            });
        });
    });
}

#[test]
fn column_grow_middle() {
    run!({
        yakui_widgets::column(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::expanded(|| {
                yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            });
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
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
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_start() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Start;
        row.show(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_center() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Center;
        row.show(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_end() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::End;
        row.show(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_stretch() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Stretch;
        row.show(|| {
            yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
            yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
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
            yakui_widgets::row(|| {
                yakui_widgets::colored_box(Color3::RED, [50.0, 50.0]);
                yakui_widgets::colored_box(Color3::GREEN, [50.0, 50.0]);
                yakui_widgets::colored_box(Color3::BLUE, [50.0, 50.0]);
            });
        });
    });
}

#[test]
fn align_center() {
    run!({
        yakui_widgets::center(|| {
            rect_50x50();
        });
    });
}

#[test]
fn align_bottom_right() {
    run!({
        yakui_widgets::align(Alignment::BOTTOM_RIGHT, || {
            rect_50x50();
        });
    });
}

fn rect<V: Dim>(w: V, h: V) {
    yakui_widgets::colored_box(Color3::WHITE, [w.to_f32(), h.to_f32()]);
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
