use yakui::widgets::{List, Pad, Slider};
use yakui::{button, checkbox, label, pad, row, slider, textbox, use_state};

pub fn run() {
    let checked = use_state(|| false);
    let name = use_state(|| String::from("Hello"));
    let step_size = use_state(|| 0.0);
    let sliding = use_state(|| 50.0);

    pad(Pad::all(20.0), || {
        List::column().item_spacing(8.0).show(|| {
            if button("Button").clicked {
                println!("Button clicked");
            }

            let res = checkbox(checked.get());
            checked.set(res.checked);

            let res = textbox(name.borrow().clone());
            if let Some(new_text) = res.into_inner().text {
                name.set(new_text);
            }

            row(|| {
                if let Some(new_step_size) = slider(step_size.get(), 0.0, 1.0).value {
                    step_size.set(new_step_size);
                }

                label(format!("Step size: {:.2}", step_size.get()));
            });

            row(|| {
                let res = Slider::new(sliding.get(), 0.0, 100.0)
                    .step(step_size.get())
                    .show();
                if let Some(new_value) = res.value {
                    sliding.set(new_value);
                }

                label(format!("Value: {:.2}", sliding.get()));
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
