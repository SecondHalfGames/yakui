//! This example shows how to use the `Canvas` widget, a low-level way to access
//! yakui's paint context from any widget.
//!
//! Eventually, this API will likely be replaced with a much friendlier API.

use yakui::paint::PaintRect;
use yakui::{canvas, center, constrained, Color, Constraints, Rect, Vec2, Vec4};

const RES: usize = 100;

pub fn run() {
    let constraints = Constraints::tight([400.0, 400.0].into());

    center(|| {
        constrained(constraints, || {
            canvas(|ctx| {
                let layout = ctx.layout.get(ctx.dom.current()).unwrap();

                for x in 0..RES {
                    for y in 0..RES {
                        let offset = Vec2::new(
                            x as f32 * layout.rect.size().x / RES as f32,
                            y as f32 * layout.rect.size().y / RES as f32,
                        );
                        let rect = Rect::from_pos_size(
                            layout.rect.pos() + offset,
                            layout.rect.size() / RES as f32,
                        );

                        let mut rect = PaintRect::new(rect);

                        let r = x as f32 / RES as f32;
                        let g = y as f32 / RES as f32;
                        rect.color = Color::from_linear(Vec4::new(r, g, 0.0, 1.0));

                        rect.add(ctx.paint);
                    }
                }
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
