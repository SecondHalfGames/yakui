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
                List::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .show(|| {
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
                                let name = use_state(|| String::from("Hello"));

                                let res = textbox(name.borrow().clone());
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
