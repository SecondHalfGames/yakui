use yakui::MainAxisSize;
use yakui_core::geometry::Color3;
use yakui_core::Alignment;
use yakui_test::{run, Test};
use yakui_widgets::widgets::List;

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
