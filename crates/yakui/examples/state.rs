use yakui::{button, center, column, label, use_state};

pub fn run() {
    let value = use_state(|| 0);

    center(|| {
        column(|| {
            label(format!("Value: {}", value.get()));

            if button("Increment").clicked {
                *value.borrow_mut() += 1;
            }
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
