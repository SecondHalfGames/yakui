use yakui::widgets::{List, Panel};
use yakui::{button, center, column, expanded, label, row, textbox, use_state, CrossAxisAlignment};

pub fn run() {
    column(|| {
        let panel = Panel::top_bottom();
        panel.show(|| {
            center(|| {
                label("Yakui Game Editor Demo");
            });
        });

        expanded(|| {
            let panel = Panel::side();
            panel.show(|| {
                let mut column = List::column();
                column.cross_axis_alignment = CrossAxisAlignment::Start;

                column.show(|| {
                    row(|| {
                        expanded(|| {
                            label("Label Label");
                        });
                        button("Button!");
                    });

                    row(|| {
                        label("More labels!");
                        button("Buttons!!!");
                    });

                    row(|| {
                        label("Input");
                        expanded(|| {
                            let name = use_state(|| String::new());

                            let res = textbox("Hello");
                            if let Some(new_text) = res.into_inner().text {
                                name.set(new_text);
                            }
                        });
                    });

                    row(|| {
                        expanded(|| {
                            button("Wide Button!");
                        });
                    });
                });
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
