use yakui::{ButtonProps, Color3, PadProps};

pub fn app(_time: f32) {
    yakui::column(|| {
        let res = yakui::button(ButtonProps::styled([70.0, 30.0]));
        if res.clicked {
            println!("Clicked the first button!");
        }

        yakui::colored_box(Color3::REBECCA_PURPLE, || {
            let padding = PadProps::even(8.0);
            yakui::pad(padding, || {
                yakui::row(|| {
                    yakui::button(ButtonProps::styled([40.0, 60.0]));
                    yakui::button(ButtonProps::styled([40.0, 60.0]));
                    yakui::button(ButtonProps::styled([40.0, 60.0]));
                });
            });
        });

        yakui::button(ButtonProps::styled([20.0, 50.0]));
    });
}
