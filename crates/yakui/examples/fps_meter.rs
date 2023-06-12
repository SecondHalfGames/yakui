//! This example shows how you might use `use_state` to keep a windowed average
//! of the current framerate of your application.

use std::collections::VecDeque;
use std::time::Instant;

use yakui::{column, label, use_state};

pub fn run() {
    let now = use_state(Instant::now);
    let new_now = Instant::now();
    let delta = new_now - now.get();
    now.set(new_now);

    let window = use_state(VecDeque::new);
    let avg = {
        let mut window = window.borrow_mut();
        window.push_back(delta.as_secs_f32());

        while window.len() > 120 {
            window.pop_front();
        }

        window.iter().sum::<f32>() / window.len() as f32
    };

    column(|| {
        label(format!("{:.2} ms", avg * 1000.0));
        label(format!("{:.0} FPS", 1.0 / avg));
    });
}

fn main() {
    bootstrap::start(run as fn());
}
