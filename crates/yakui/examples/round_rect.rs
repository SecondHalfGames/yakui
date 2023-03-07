use yakui::{center, Vec2};

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    let radius = 25.0 + 25.0 * state.time.cos();

    center(|| {
        let mut rect = yakui::widgets::RoundRect::new(radius);
        rect.min_size = Vec2::new(100.0, 100.0);
        rect.show();
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}
