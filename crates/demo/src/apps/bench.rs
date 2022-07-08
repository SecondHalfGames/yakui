use yakui::{ButtonProps, Color3};

pub fn app(_time: f32) {
    let colors = [Color3::RED, Color3::GREEN, Color3::BLUE];

    yakui::row(|| {
        for x in 0..100 {
            yakui::column(|| {
                for y in 0..100 {
                    let color = colors[(x + y) % colors.len()];

                    let w = 2.0 + 3.0 + (x / 2) as f32;
                    let h = 2.0 + 3.0 + (y / 2) as f32;
                    yakui::button(ButtonProps {
                        fill: color,
                        ..ButtonProps::styled([w, h])
                    });
                }
            });
        }
    });
}
