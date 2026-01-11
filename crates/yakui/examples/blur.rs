use bootstrap::ExampleState;
use yakui::widgets::CutOut;
use yakui::{draggable, image, offset, text, use_state, Color, Constraints, Vec2};
use yakui_core::CrossAxisAlignment;
use yakui_widgets::constrained;
use yakui_widgets::widgets::{List, Pad};

pub fn run(state: &mut ExampleState) {
    let pos = use_state(|| Vec2::ZERO);

    image(state.monkey, Vec2::splat(100.0));

    offset(pos.get(), || {
        let res = draggable(|| {
            let constraints = Constraints {
                min: Vec2::new(400.0, 0.0),
                max: Vec2::new(400.0, f32::INFINITY),
            };

            constrained(constraints, || {
                List::column()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .show(|| {
                        CutOut::new(state.monkey_blurred, Color::hex(0x5cc9ff).with_alpha(0.25))
                            .radius(25.0)
                            .show_children(|| {
                                Pad::all(16.0).show(|| {
                                    text(48.0, "Blur Demo");
                                });
                            });

                        CutOut::new(state.monkey_blurred, Color::hex(0x444444).with_alpha(0.1))
                            .show_children(|| {
                                Pad::all(16.0).show(|| {
                                    text(24.0, "Lorem ipsum dolor sit amet\n".repeat(5));
                                });
                            });
                    });
            });
        });

        if let Some(new) = res.dragging {
            pos.set(new.current);
        }
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}
