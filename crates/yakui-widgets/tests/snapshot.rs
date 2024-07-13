use yakui::{Constraints, CrossAxisAlignment, Dim2, MainAxisAlignment, MainAxisSize, Vec2};
use yakui_core::geometry::Color;
use yakui_core::{Alignment, Pivot};
use yakui_test::{run, Test};
use yakui_widgets::widgets::{Button, List, Pad, UnconstrainedBox};
use yakui_widgets::{
    align, button, center, checkbox, colored_box, colored_box_container, column, constrained,
    expanded, pad, reflow, row, text,
};

#[test]
fn colored_box_basic() {
    run!({
        colored_box(Color::RED, [50.0, 50.0]);
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
fn column_main_align_end() {
    run!({
        let mut container = List::column();
        container.main_axis_alignment = MainAxisAlignment::End;
        container.show(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn column_main_align_center() {
    run!({
        let mut container = List::column();
        container.main_axis_alignment = MainAxisAlignment::Center;
        container.show(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn column_item_spacing() {
    run!({
        let mut container = List::column();
        container.item_spacing = 10.0;
        container.show(|| {
            rect_50x50();
            rect_50x50();
            rect_50x50();
        });
    });
}

#[test]
fn row_item_spacing() {
    run!({
        let mut container = List::row();
        container.item_spacing = 10.0;
        container.show(|| {
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
                colored_box(Color::RED, [50.0, 50.0]);
            });
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_middle() {
    run!({
        row(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color::GREEN, [50.0, 50.0]);
            });
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_grow_last() {
    run!({
        row(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color::BLUE, [50.0, 50.0]);
            });
        });
    });
}

#[test]
fn row_grow_middle_spacing() {
    run!({
        let mut container = List::row();
        container.item_spacing = 10.0;
        container.show(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color::GREEN, [50.0, 50.0]);
            });
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn column_grow_middle() {
    run!({
        column(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            expanded(|| {
                colored_box(Color::GREEN, [50.0, 50.0]);
            });
            colored_box(Color::BLUE, [50.0, 50.0]);
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
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_start() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Start;
        row.show(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_center() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Center;
        row.show(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_end() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::End;
        row.show(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
        });
    });
}

#[test]
fn row_cross_stretch() {
    run!({
        let mut row = List::row();
        row.cross_axis_alignment = CrossAxisAlignment::Stretch;
        row.show(|| {
            colored_box(Color::RED, [50.0, 50.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
            colored_box(Color::BLUE, [50.0, 50.0]);
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
                colored_box(Color::RED, [50.0, 50.0]);
                colored_box(Color::GREEN, [50.0, 50.0]);
                colored_box(Color::BLUE, [50.0, 50.0]);
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
fn align_text_center() {
    run!({
        center(|| {
            text(60.0, "X X X X X");
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
            colored_box_container(Color::RED, || {
                pad(padding, || {
                    colored_box(Color::GREEN, Vec2::splat(50.0));
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
            colored_box_container(Color::RED, || {
                pad(padding, || {});
            });
        });
    });
}

#[test]
fn button_text_alignment() {
    run!({
        let constraints = Constraints::tight(Vec2::splat(800.0));
        align(Alignment::TOP_LEFT, || {
            constrained(constraints, || {
                let mut button = Button::styled("X X X X X");
                button.style.text.font_size = 60.0;
                button.show();
            });
        });
    });
}

#[test]
fn constrain_checkbox() {
    run!({
        let constraints = Constraints::tight(Vec2::splat(100.0));

        center(|| {
            constrained(constraints, || {
                checkbox(true);
            });
        });
    });
}

#[test]
fn column_intrinsic() {
    run!({
        column(|| {
            button("X X X");
        });
    });
}

#[test]
fn row_reflow() {
    run!({
        align(Alignment::TOP_LEFT, || {
            let mut list = List::row();
            list.main_axis_size = MainAxisSize::Min;
            list.show(|| {
                colored_box(Color::RED, [50.0, 50.0]);
                colored_box(Color::GREEN, [50.0, 50.0]);

                reflow(Alignment::BOTTOM_RIGHT, Pivot::TOP_LEFT, Dim2::ZERO, || {
                    colored_box(Color::BLUE, [100.0, 50.0]);
                });

                reflow(
                    Alignment::BOTTOM_RIGHT,
                    Pivot::TOP_LEFT,
                    Dim2::pixels(0.0, 50.0),
                    || {
                        colored_box(Color::WHITE, [100.0, 100.0]);
                    },
                );
            });
        });
    });
}

fn rect<V: IntoF32>(w: V, h: V) {
    colored_box(Color::WHITE, [w.to_f32(), h.to_f32()]);
}

fn rect_50x50() {
    rect(50.0, 50.0);
}

trait IntoF32 {
    fn to_f32(self) -> f32;
}

impl IntoF32 for f32 {
    fn to_f32(self) -> f32 {
        self
    }
}

impl IntoF32 for i32 {
    fn to_f32(self) -> f32 {
        self as f32
    }
}
