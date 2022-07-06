use yakui::{ButtonProps, Color3};

pub fn app(time: f32) {
    yakui::vertical(|| {
        yakui::horizontal(|| {
            let x = 50.0 * time.sin();
            let y = 20.0 * (time + 1.0).sin();

            yakui::button(ButtonProps {
                fill: Color3::rgb(127, 90, 200),
                hover_fill: Some(Color3::rgb(100, 60, 150)),
                down_fill: Some(Color3::rgb(70, 40, 110)),
                ..ButtonProps::new([70.0, 30.0])
            });

            yakui::fsbox([100.0 + x, 100.0 + y], Color3::RED);
            yakui::fsbox([40.0, 30.0], Color3::GREEN);
            yakui::fsbox([60.0, 40.0], Color3::BLUE);
        });

        if yakui::fsbox([200.0, 100.0], Color3::REBECCA_PURPLE).clicked {
            println!("Clicked it!");
        }
    });
}
