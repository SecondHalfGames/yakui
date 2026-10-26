use yakui::{constrained, row, text, Constraints, Vec2};

// Demonstrates bug https://github.com/SecondHalfGames/yakui/issues/104
pub fn run() {
    constrained(Constraints::tight(Vec2::splat(100.0)), || {
        row(|| {
            text(
                32.0,
                "Lorem ipsum dolor sit amet blah blah blah I just want some long text here",
            );
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
