use yakui::ButtonProps;

pub fn app(time: f32) {
    yakui::vertical(|| {
        let res = yakui::button(ButtonProps::styled([70.0, 30.0]));
        if res.clicked {
            println!("Clicked the first button!");
        }

        yakui::horizontal(|| {
            yakui::button(ButtonProps::styled([40.0, 60.0]));
            yakui::button(ButtonProps::styled([40.0, 60.0]));
            yakui::button(ButtonProps::styled([40.0, 60.0]));
        });

        yakui::button(ButtonProps::styled([20.0, 50.0]));
    });
}
