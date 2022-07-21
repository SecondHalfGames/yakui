use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::row(|| {
        let panel = yakui::widgets::Panel::side();

        panel.show(|| {
            yakui::column(|| {
                yakui::row(|| {
                    yakui::label("Label Label");
                    yakui::button("Button!");
                });

                yakui::row(|| {
                    yakui::label("More labels!");
                    yakui::button("Buttons!!!");
                });

                yakui::button("Wide Button!");
            });
        });
    });
}
