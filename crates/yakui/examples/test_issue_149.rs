use yakui::widgets::Scrollable;
use yakui::{
    button, column, constrained, label, reflow, row, use_state, Alignment, Constraints, Dim2,
    Pivot, Vec2,
};

// Demonstrates bug https://github.com/SecondHalfGames/yakui/issues/149
pub fn run() {
    let buttons = ["Hello", "World", "Foobar", "Aaaggh", "Badonka", "Hachinka"];
    let open = use_state(|| true);

    constrained(Constraints::loose(Vec2::new(f32::INFINITY, 138.0)), || {
        Scrollable::vertical().show(|| {
            column(|| {
                row(|| {
                    button("pushing things to the side");

                    column(|| {
                        if button("toggle list").clicked {
                            open.modify(|v| !v);
                        }

                        if open.get() {
                            reflow(Alignment::BOTTOM_LEFT, Pivot::TOP_LEFT, Dim2::ZERO, || {
                                constrained(
                                    Constraints::loose(Vec2::new(f32::INFINITY, 160.0)),
                                    || {
                                        Scrollable::vertical().show(|| {
                                            column(|| {
                                                for option in buttons.iter() {
                                                    button(*option);
                                                }
                                            });
                                        });
                                    },
                                );
                            });
                        }
                    });
                });

                label("the text below is gonna be\n clipped aaaaa                                  oh hi");
                label("this text\nis gonna\noverflow aaaa");
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
