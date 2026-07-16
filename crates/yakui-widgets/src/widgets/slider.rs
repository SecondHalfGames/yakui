use std::cell::Cell;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::input::KeyCode;
use yakui_core::widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::{auto_builders, colored_circle, colors, draggable, util};

use crate::colored_box;

const TRACK_COLOR: Color = colors::BACKGROUND_3;
const KNOB_COLOR: Color = colors::TEXT_MUTED;

const DEFAULT_WIDTH: f32 = 150.0;
const TRACK_HEIGHT: f32 = 10.0;
const KNOB_SIZE: f32 = 24.0;
const TOTAL_HEIGHT: f32 = KNOB_SIZE * 1.5;

#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Slider {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

auto_builders!(Slider {
    value: f64,
    min: f64,
    max: f64,
    step: Option<f64>,
});

impl Slider {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        Slider {
            value,
            min,
            max,
            step: None,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<SliderResponse> {
        util::widget::<SliderWidget>(self)
    }
}

#[derive(Debug)]
pub struct SliderResponse {
    pub confirmed: bool,
    pub value: Option<f64>,
    pub focused: bool,
}

#[derive(Debug)]
pub struct SliderWidget {
    props: Slider,
    rect: Cell<Option<Rect>>,
    dragging: bool,
    pending_value: Option<f64>,
    confirmed: bool,
    focused: bool,
}

impl SliderWidget {
    fn current_value(&self) -> f64 {
        self.pending_value.unwrap_or(self.props.value)
    }

    fn keyboard_step(&self) -> f64 {
        self.props
            .step
            // 1% if no step configured
            .unwrap_or((self.props.max - self.props.min) / 100.0)
            .abs()
    }

    fn clamp_value(&self, value: f64) -> f64 {
        value.clamp(
            self.props.min.min(self.props.max),
            self.props.min.max(self.props.max),
        )
    }
}

impl Widget for SliderWidget {
    type Props<'a> = Slider;
    type Response = SliderResponse;

    fn new() -> Self {
        Self {
            props: Slider::new(0.0, 0.0, 1.0),
            rect: Cell::new(None),
            dragging: false,
            pending_value: None,
            confirmed: false,
            focused: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        colored_box(TRACK_COLOR, [0.0, TRACK_HEIGHT]);
        let res = draggable(|| {
            colored_circle(KNOB_COLOR, KNOB_SIZE);
        });

        let confirmed = (self.dragging && res.dragging.is_none()) || self.confirmed;
        self.confirmed = false;
        self.dragging = res.dragging.is_some();

        let mut value = self.props.value;

        if let (Some(drag), Some(rect)) = (res.dragging, self.rect.get()) {
            let min_pos = rect.pos().x;
            let max_pos = rect.pos().x + rect.size().x - KNOB_SIZE;
            let actual_pos = drag.current.x.clamp(min_pos, max_pos);

            let percentage = (actual_pos - min_pos) / (max_pos - min_pos);
            value = self.props.min + percentage as f64 * (self.props.max - self.props.min);
        }

        if let Some(pending_value) = self.pending_value.take() {
            value = pending_value;
        }

        if let Some(step) = self.props.step {
            value = round_to_step(value, step);
        }
        value = self.clamp_value(value);

        if value != self.props.value {
            SliderResponse {
                value: Some(value),
                confirmed,
                focused: self.focused,
            }
        } else {
            SliderResponse {
                value: None,
                confirmed,
                focused: self.focused,
            }
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

    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::SEQUENTIAL | FocusPolicy::DIRECTIONAL | FocusPolicy::POINTER
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::FocusChanged(focused) => {
                self.focused = *focused;
                EventResponse::Bubble
            }
            WidgetEvent::KeyChanged { key, down, .. } => {
                let value = match key {
                    KeyCode::ArrowLeft => self.current_value() - self.keyboard_step(),
                    KeyCode::ArrowRight => self.current_value() + self.keyboard_step(),
                    KeyCode::Home => self.props.min,
                    KeyCode::End => self.props.max,
                    _ => return EventResponse::Bubble,
                };

                if *down {
                    self.pending_value = Some(self.clamp_value(value));
                    self.confirmed = true;
                }

                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
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
