use bootstrap::ExampleState;
use yakui::colors::BACKGROUND_2;
use yakui::navigation::NavDirections;
use yakui::widgets::{List, Pad, Slider, TextBox, Trap};
use yakui::{
    button, checkbox, colored_box_container, label, pad, scroll_vertical, use_state,
    MainAxisAlignment, MainAxisSize,
};

pub fn run(_state: &mut ExampleState) {
    pad(Pad::all(20.0), || {
        List::column()
            .item_spacing(8.0_f32)
            .main_axis_size(MainAxisSize::Min)
            .show(|| {
                label("Start by clicking, or using sequential navigation");

                section("Directional grid", || {
                    List::column().item_spacing(8.0_f32).show(|| {
                        for row in 0..3 {
                            List::row().item_spacing(8.0_f32).show(|| {
                                for column in 0..3 {
                                    let index = row * 3 + column + 1;
                                    button(format!("Button {index}"));
                                }
                            });
                        }
                    });
                });

                section(
                    "Widgets, like the text box and slider, might consume navigation keys",
                    || {
                        let checked = use_state(|| false);
                        let checkbox_res = checkbox(checked.get());
                        checked.set(checkbox_res.checked);

                        let text = use_state(|| String::from("Edit me"));
                        let textbox_res = TextBox::new(text.borrow().clone())
                            .placeholder("Type here")
                            .show();
                        if let Some(new_text) = textbox_res.into_inner().text {
                            text.set(new_text);
                        }

                        let slider_value = use_state(|| 50.0);
                        let slider_res =
                            Slider::new(slider_value.get(), 0.0, 100.0).step(5.0).show();
                        if let Some(value) = slider_res.value {
                            slider_value.set(value);
                        }
                        label(format!("Slider value: {:.0}", slider_value.get()));
                    },
                );

                Trap::new(NavDirections::DIRECTIONAL).show(|| {
                    section(
                        "Directional trap! Arrow navigation stays inside this row.",
                        || {
                            button("You're trapped! 1");
                            button("You're trapped! 2");
                            button("You're trapped! 3");
                        },
                    );
                });

                section("Scrollable focus", || {
                    scroll_vertical(|| {
                        List::column().item_spacing(4.0_f32).show(|| {
                            for index in 1..=12 {
                                button(format!("Scrollable button {index}"));
                            }
                        });
                    });
                });
            });
    });
}

fn section(title: &'static str, children: impl FnOnce()) {
    List::column()
        .main_axis_size(MainAxisSize::Min)
        .item_spacing(4.0_f32)
        .show(|| {
            label(title);

            colored_box_container(BACKGROUND_2, || {
                Pad::all(8.0).show(|| {
                    List::row()
                        .main_axis_size(MainAxisSize::Min)
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .item_spacing(8.0_f32)
                        .show(|| {
                            children();
                        });
                });
            });
        });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}
