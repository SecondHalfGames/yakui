use std::cell::Cell;

use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::{colored_circle, colors, draggable, util};

use crate::colored_box;

const TRACK_COLOR: Color = colors::BACKGROUND_3;
const KNOB_COLOR: Color = colors::TEXT_MUTED;

const DEFAULT_WIDTH: f32 = 150.0;
const TRACK_HEIGHT: f32 = 10.0;
const KNOB_SIZE: f32 = 24.0;
const TOTAL_HEIGHT: f32 = KNOB_SIZE * 1.5;

#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Slider {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

impl Slider {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        Slider {
            value,
            min,
            max,
            step: None,
        }
    }

    pub fn show(self) -> Response<SliderResponse> {
        util::widget::<SliderWidget>(self)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct SliderResponse {
    pub value: Option<f64>,
}

#[derive(Debug)]
pub struct SliderWidget {
    props: Slider,
    rect: Cell<Option<Rect>>,
}

impl Widget for SliderWidget {
    type Props<'a> = Slider;
    type Response = SliderResponse;

    fn new() -> Self {
        Self {
            props: Slider::new(0.0, 0.0, 1.0),
            rect: Cell::new(None),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        colored_box(TRACK_COLOR, [0.0, TRACK_HEIGHT]);
        let res = draggable(|| {
            colored_circle(KNOB_COLOR, KNOB_SIZE);
        });

        let mut value = self.props.value;

        if let (Some(drag), Some(rect)) = (res.dragging, self.rect.get()) {
            let min_pos = rect.pos().x;
            let max_pos = rect.pos().x + rect.size().x - KNOB_SIZE;
            let actual_pos = drag.current.x.clamp(min_pos, max_pos);

            let percentage = (actual_pos - min_pos) / (max_pos - min_pos);
            value = self.props.min + percentage as f64 * (self.props.max - self.props.min);
        }

        if let Some(step) = self.props.step {
            value = round_to_step(value, step);
        }

        if value != self.props.value {
            SliderResponse { value: Some(value) }
        } else {
            SliderResponse { value: None }
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let size = Vec2::new(
            constraints.constrain_width(DEFAULT_WIDTH).max(KNOB_SIZE),
            constraints.min.y.max(TOTAL_HEIGHT),
        );

        let track = node.children[0];
        let knob = node.children[1];

        let track_constraints = Constraints::tight(Vec2::new(size.x - KNOB_SIZE, TRACK_HEIGHT));
        ctx.calculate_layout(track, track_constraints);
        ctx.layout.set_pos(
            track,
            Vec2::new(KNOB_SIZE / 2.0, (TOTAL_HEIGHT - TRACK_HEIGHT) / 2.0),
        );

        let percentage = (self.props.value - self.props.min) / (self.props.max - self.props.min);
        let percentage = percentage.clamp(0.0, 1.0);
        let knob_offset = (size.x - KNOB_SIZE) * percentage as f32;
        let knob_pos = Vec2::new(knob_offset, (TOTAL_HEIGHT - KNOB_SIZE) / 2.0);
        ctx.calculate_layout(knob, Constraints::none());
        ctx.layout.set_pos(knob, knob_pos);

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        // This is a little gross: stash our position from this frame's layout
        // pass so that we can compare it against any drag updates that happen
        // at the beginning of the next frame.
        let layout = ctx.layout.get(ctx.dom.current()).unwrap();
        self.rect.set(Some(layout.rect));

        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }
    }
}

fn round_to_step(value: f64, step: f64) -> f64 {
    if step == 0.0 {
        value
    } else {
        (value / step).round() * step
    }
}
