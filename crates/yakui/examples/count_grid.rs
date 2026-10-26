use yakui::{Color, Vec2};
use yakui_core::geometry::Constraints;
use yakui_core::{
    Alignment, CrossAxisAlignment, Direction, MainAxisAlignItems, MainAxisAlignment, MainAxisSize,
    Response,
};
use yakui_widgets::widgets::{CountGrid, List, StateResponse};
use yakui_widgets::{
    align, button, center, checkbox, colored_box_container, constrained, label, row, use_state,
};

pub fn run() {
    let main_axis_size = use_state(|| MainAxisSize::Min);
    let main_axis_alignment = use_state(|| MainAxisAlignment::Start);
    let main_axis_align_items = use_state(|| MainAxisAlignItems::Start);
    let cross_axis_alignment = use_state(|| CrossAxisAlignment::Start);
    let direction = use_state(|| Direction::Down);
    let compare_with_list = use_state(|| false);

    column_spacing(|| {
        parameter_select(
            &main_axis_size,
            &main_axis_alignment,
            &main_axis_align_items,
            &cross_axis_alignment,
            &direction,
        );

        label("The following 3x2 grid will be layed out in a 600x400 container");
        test_window(|| {
            let mut g = CountGrid::row(2);
            g.direction = *direction.borrow();
            g.main_axis_alignment = *main_axis_alignment.borrow();
            g.main_axis_size = *main_axis_size.borrow();
            g.main_axis_align_items = *main_axis_align_items.borrow();
            g.cross_axis_alignment = *cross_axis_alignment.borrow();
            g.show(|| {
                box_with_label(Color::RED, 50.0);
                box_with_label(Color::GREEN, 50.0);
                box_with_label(Color::BLUE, 70.0);
                box_with_label(Color::CORNFLOWER_BLUE, 80.0);
                box_with_label(Color::REBECCA_PURPLE, 30.0);
                box_with_label(Color::FUCHSIA, 70.0);
            });
        });

        row(|| {
            *compare_with_list.borrow_mut() = checkbox(compare_with_list.get()).checked;
            label("compare with list");
        });
        if compare_with_list.get() {
            test_window(|| {
                let mut l = List::row();
                l.direction = *direction.borrow();
                l.cross_axis_alignment = *cross_axis_alignment.borrow();
                l.main_axis_alignment = *main_axis_alignment.borrow();
                l.main_axis_size = *main_axis_size.borrow();
                l.show(|| {
                    box_with_label(Color::RED, 50.0);
                    box_with_label(Color::BLUE, 70.0);
                    box_with_label(Color::REBECCA_PURPLE, 30.0);
                });
            });
        }
    });
}

fn test_window(children: impl FnOnce()) {
    constrained(Constraints::tight(Vec2::new(600.0, 400.0)), || {
        colored_box_container(Color::rgb(0, 0, 0), || {
            align(Alignment::TOP_LEFT, || {
                constrained(Constraints::loose(Vec2::new(600.0, 400.0)), || {
                    colored_box_container(Color::GRAY, children);
                });
            });
        });
    });
}

fn parameter_select(
    main_axis_size: &Response<StateResponse<MainAxisSize>>,
    main_axis_alignment: &Response<StateResponse<MainAxisAlignment>>,
    main_axis_align_items: &Response<StateResponse<MainAxisAlignItems>>,
    cross_axis_alignment: &Response<StateResponse<CrossAxisAlignment>>,
    direction: &Response<StateResponse<Direction>>,
) {
    row_spacing(|| {
        label("Main axis size:");
        let main_size_button = |text, size| {
            if main_axis_size.get() == size {
                label(text);
                return;
            }
            if button(text).clicked {
                *main_axis_size.borrow_mut() = size;
            }
        };
        main_size_button("Min", MainAxisSize::Min);
        main_size_button("Max", MainAxisSize::Max);
    });
    row_spacing(|| {
        label("Main axis alignment:");
        let main_align_button = |text, alignment| {
            if main_axis_alignment.get() == alignment {
                label(text);
                return;
            }
            if button(text).clicked {
                *main_axis_alignment.borrow_mut() = alignment;
            }
        };
        main_align_button("Start", MainAxisAlignment::Start);
        main_align_button("Center", MainAxisAlignment::Center);
        main_align_button("End", MainAxisAlignment::End);
    });
    row_spacing(|| {
        label("Main axis align items:");
        let main_align_items_button = |text, alignment| {
            if main_axis_align_items.get() == alignment {
                label(text);
                return;
            }
            if button(text).clicked {
                *main_axis_align_items.borrow_mut() = alignment;
            }
        };
        main_align_items_button("Start", MainAxisAlignItems::Start);
        main_align_items_button("Center", MainAxisAlignItems::Center);
        main_align_items_button("End", MainAxisAlignItems::End);
        main_align_items_button("Stretch", MainAxisAlignItems::Stretch);
    });
    row_spacing(|| {
        label("Cross axis alignment:");
        let cross_align_button = |text, alignment| {
            if cross_axis_alignment.get() == alignment {
                label(text);
                return;
            }
            if button(text).clicked {
                *cross_axis_alignment.borrow_mut() = alignment;
            }
        };
        cross_align_button("Start", CrossAxisAlignment::Start);
        cross_align_button("Center", CrossAxisAlignment::Center);
        cross_align_button("End", CrossAxisAlignment::End);
        cross_align_button("Stretch", CrossAxisAlignment::Stretch);
    });
    row_spacing(|| {
        label("Direction:");
        let direction_button = |text, dir| {
            if *direction.borrow() == dir {
                label(text);
                return;
            }
            if button(text).clicked {
                *direction.borrow_mut() = dir;
            }
        };
        direction_button("Down", Direction::Down);
        direction_button("Right", Direction::Right);
    });
}

fn column_spacing<F: FnOnce()>(children: F) {
    List::column().item_spacing(5.0).show(children);
}

fn row_spacing<F: FnOnce()>(children: F) {
    List::row().item_spacing(5.0).show(children);
}

fn box_with_label(color: Color, size: f32) {
    constrained(Constraints::tight(Vec2::splat(size)), || {
        colored_box_container(color, || {
            center(|| {
                yakui::text(size / 3.0, format!("{size}x{size}"));
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
