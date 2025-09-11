use yakui::{center, column, row, spacer, text, Color, Vec2};

use bootstrap::ExampleState;
use yakui_widgets::border_radius::BorderRadius;

pub fn run(state: &mut ExampleState) {
    center(|| {
        column(|| {
            text(32.0, "Individual Corner Radii Demo");
            spacer(1);

            row(|| {
                let mut rect = yakui::widgets::RoundRect::new(10.0);
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::CYAN;
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Uniform");
                    });
                });

                spacer(1);

                let mut rect = yakui::widgets::RoundRect::new((20.0, 5.0, 20.0, 5.0));
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::YELLOW;
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Custom");
                    });
                });
            });

            spacer(1);

            row(|| {
                let mut rect = yakui::widgets::RoundRect::new(BorderRadius::top(15.0));
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::RED;
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Top");
                    });
                });

                spacer(1);

                let mut rect = yakui::widgets::RoundRect::new(BorderRadius::right(15.0));
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::GREEN;
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Right");
                    });
                });
            });

            spacer(1);

            row(|| {
                let mut radius = BorderRadius::uniform(10.0);
                radius.top_left = 30.0;

                let mut rect = yakui::widgets::RoundRect::new(radius);
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::rgb(255, 0, 255);
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Mixed");
                    });
                });

                spacer(1);

                let animated_radius = 5.0 + 15.0 * (state.time * 2.0).sin().abs();
                let mut rect =
                    yakui::widgets::RoundRect::new((animated_radius, 5.0, animated_radius, 5.0));
                rect.min_size = Vec2::new(100.0, 60.0);
                rect.color = Color::BLUE;
                rect.show_children(|| {
                    center(|| {
                        text(16.0, "Animated");
                    });
                });
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}
