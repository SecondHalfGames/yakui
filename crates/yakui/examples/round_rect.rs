use yakui::{center, column, row, spacer, text, Color, Vec2};

use bootstrap::ExampleState;
use yakui_widgets::border::{Border, BorderRadius};

pub fn run(state: &mut ExampleState) {
    center(|| {
        column(|| {
            text(32.0, "Round Rect Demo");
            spacer(1);

            for row_idx in 0..5 {
                row(|| {
                    for col_idx in 0..5 {
                        let cell_idx = row_idx * 5 + col_idx;

                        if col_idx > 0 {
                            spacer(1);
                        }

                        create_round_rect(cell_idx, state);
                    }
                });

                if row_idx < 4 {
                    spacer(1);
                }
            }
        });
    });
}

fn create_round_rect(index: usize, state: &ExampleState) {
    let (radius, color) = match index {
        0 => (BorderRadius::uniform(10.0), Color::CYAN),
        1 => (BorderRadius::from((20.0, 5.0, 20.0, 5.0)), Color::YELLOW),
        2 => (BorderRadius::top(15.0), Color::RED),
        3 => (BorderRadius::right(15.0), Color::GREEN),
        4 => (
            BorderRadius::uniform(10.0).top_left(30.0),
            Color::rgb(255, 0, 255),
        ),
        5 => {
            let animated_radius = 15.0 * (state.time * 2.0).sin().abs();
            (
                BorderRadius::from((animated_radius, 5.0, animated_radius, 5.0)),
                Color::BLUE,
            )
        }
        6 => (
            BorderRadius::from((25.0, 10.0, 5.0, 15.0)),
            Color::rgb(255, 128, 0),
        ),
        7 => (
            BorderRadius::from((0.0, 20.0, 0.0, 20.0)),
            Color::rgb(128, 255, 128),
        ),
        8 => (
            BorderRadius::from((30.0, 0.0, 30.0, 0.0)),
            Color::rgb(255, 192, 203),
        ),
        9 => (
            BorderRadius::from((15.0, 15.0, 25.0, 5.0)),
            Color::rgb(173, 216, 230),
        ),
        10 => (
            BorderRadius::from((8.0, 22.0, 12.0, 18.0)),
            Color::rgb(221, 160, 221),
        ),

        11 => (
            BorderRadius::from((18.0, 18.0, 0.0, 0.0)),
            Color::rgb(255, 215, 0),
        ),
        12 => (
            BorderRadius::from((0.0, 0.0, 25.0, 25.0)),
            Color::rgb(255, 99, 71),
        ),
        13 => (
            BorderRadius::from((35.0, 5.0, 5.0, 35.0)),
            Color::rgb(144, 238, 144),
        ),
        14 => (
            BorderRadius::from((12.0, 8.0, 20.0, 16.0)),
            Color::rgb(175, 238, 238),
        ),
        15 => (
            BorderRadius::from((20.0, 20.0, 20.0, 0.0)),
            Color::rgb(255, 182, 193),
        ),

        16 => (
            BorderRadius::from((28.0, 7.0, 14.0, 21.0)),
            Color::rgb(240, 230, 140),
        ),
        17 => (
            BorderRadius::from((5.0, 25.0, 5.0, 25.0)),
            Color::rgb(255, 160, 122),
        ),
        18 => (
            BorderRadius::from((0.0, 30.0, 30.0, 0.0)),
            Color::rgb(152, 251, 152),
        ),
        19 => (
            BorderRadius::from((15.0, 0.0, 15.0, 30.0)),
            Color::rgb(135, 206, 250),
        ),
        20 => (
            BorderRadius::from((22.0, 11.0, 6.0, 17.0)),
            Color::rgb(238, 130, 238),
        ),

        21 => (
            BorderRadius::from((40.0, 40.0, 0.0, 0.0)),
            Color::rgb(250, 128, 114),
        ),
        22 => (
            BorderRadius::from((10.0, 30.0, 10.0, 30.0)),
            Color::rgb(176, 196, 222),
        ),
        23 => (
            BorderRadius::from((25.0, 0.0, 0.0, 25.0)),
            Color::rgb(255, 218, 185),
        ),
        24 => (
            BorderRadius::from((3.0, 18.0, 27.0, 9.0)),
            Color::rgb(230, 230, 250),
        ),

        _ => (BorderRadius::uniform(0.0), Color::WHITE),
    };

    yakui::widgets::RoundRect::new(radius)
        .min_size(Vec2::new(80.0, 50.0))
        .color(color)
        .border(Border::new(Color::CYAN, (index as f32).min(10.0)))
        .show_children(|| {
            yakui::constrained(yakui::Constraints::tight(Vec2::new(80.0, 50.0)), || {
                yakui::pad(yakui::widgets::Pad::all(4.0), || {
                    center(|| {
                        text(12.0, format!("{}", index));
                    });
                });
            });
        });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}
