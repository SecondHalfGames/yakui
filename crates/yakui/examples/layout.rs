use yakui::widgets::Pad;
use yakui::{button, expanded, pad, Vec2};
use yakui_core::geometry::Color;
use yakui_core::{Alignment, CrossAxisAlignment, MainAxisAlignment, MainAxisSize};
use yakui_widgets::widgets::List;

pub fn run() {
    center_ui(|| {
        button("test");
        yakui::colored_box(Color::RED, Vec2::new(100.0, 100.0));
        ui_panel(|| {
            yakui::colored_box(Color::RED, Vec2::new(100.0, 100.0));
        });
        ui_panel(|| {
            yakui::colored_box(Color::GREEN, Vec2::new(100.0, 100.0));
        });
        ui_panel(|| {
            let mut l = List::row();
            l.main_axis_size = MainAxisSize::Min;
            l.show(|| {
                for i in 0..10 {
                    yakui::flexible(1, || {
                        pad(Pad::all(8.0), || {
                            yakui::colored_box(Color::BLUE, Vec2::new(20.0, 20.0));
                        });
                    });
                }
            });
        });
        let mut l = List::row();
        l.main_axis_size = MainAxisSize::Max;
        l.main_axis_alignment = MainAxisAlignment::End;
        l.show(|| {
            expanded( || {
                ui_panel(|| {
                    yakui::colored_box(Color::CORNFLOWER_BLUE, Vec2::new(100.0, 100.0));
                });
            });

            expanded(|| {
                ui_panel(|| {
                    yakui::colored_box(Color::CORNFLOWER_BLUE, Vec2::new(100.0, 100.0));
                });
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

fn center_ui<F: FnOnce()>(f: F) {
    yakui::align(Alignment::TOP_CENTER, || {
        yakui::pad(Pad::all(16.0), || {
            yakui::widgets::IntrinsicWidth::new().show(|| {
                let mut l = List::column();
                l.main_axis_size = MainAxisSize::Max;
                l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                l.show(|| {
                    yakui::spacer(1);
                    f();
                });
            });
        });
    });
}

fn ui_panel<F: FnOnce()>(f: F) {
    // yakui::flexible(1, || {
    yakui::colored_box_container(Color::rgba(255, 255, 0, 128), || {
        yakui::pad(Pad::all(8.0), || {
            f();
        });
    });
    // });
}
