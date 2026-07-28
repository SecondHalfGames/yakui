use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::input::{KeyCode, MouseButton};
use yakui_core::widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::border::Border;
use crate::shapes::RoundedRectangle;
use crate::{colors, shapes};

const OUTER_SIZE: f32 = 24.0;
const INNER_SIZE: f32 = 16.0;

/**
A checkbox with a provided value.

Responds with [CheckboxResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
let mut value = false;

value = yakui::checkbox(value).checked;
```
*/
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Checkbox {
    pub checked: bool,
}

impl Checkbox {
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }

    #[track_caller]
    pub fn show(self) -> Response<CheckboxResponse> {
        crate::util::widget::<CheckboxWidget>(self)
    }
}

#[derive(Debug)]
pub struct CheckboxWidget {
    props: Checkbox,
    hovering: bool,
    mouse_down: bool,
    just_toggled: bool,
    focused: bool,
}

#[derive(Debug)]
pub struct CheckboxResponse {
    pub checked: bool,
    pub focused: bool,
}

impl Widget for CheckboxWidget {
    type Props<'a> = Checkbox;
    type Response = CheckboxResponse;

    fn new() -> Self {
        Self {
            props: Checkbox::new(false),
            hovering: false,
            mouse_down: false,
            just_toggled: false,
            focused: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut checked = self.props.checked;
        if self.just_toggled {
            checked = !checked;
            self.just_toggled = false;
        }

        CheckboxResponse {
            checked,
            focused: self.focused,
        }
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let padding = Vec2::splat(OUTER_SIZE - INNER_SIZE);
        let mut check_rect = layout_node.rect;
        check_rect.set_pos(check_rect.pos() + padding / 2.0);
        check_rect.set_size(check_rect.size() - padding);

        let (background, border) = if self.mouse_down {
            (
                colors::BACKGROUND_3.adjust(0.8),
                Border::new(Color::WHITE, 1.0),
            )
        } else if self.hovering {
            (
                colors::BACKGROUND_3.adjust(1.2),
                Border::new(Color::WHITE.adjust(0.75), 1.0),
            )
        } else if self.focused {
            (colors::BACKGROUND_3, Border::new(Color::WHITE, 1.0))
        } else {
            (colors::BACKGROUND_3, Border::new(colors::BACKGROUND_1, 1.0))
        };
        let bg = RoundedRectangle::new(layout_node.rect, 6.0)
            .color(background)
            .border(Some(border));
        bg.add(ctx.paint);

        if self.props.checked {
            shapes::cross(ctx.paint, check_rect, colors::TEXT);
        }
    }

    fn layout(&self, _ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        constraints.constrain_min(Vec2::splat(OUTER_SIZE))
    }

    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::SEQUENTIAL | FocusPolicy::DIRECTIONAL | FocusPolicy::POINTER
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE | EventInterest::FOCUSED_KEYBOARD
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
                EventResponse::Sink
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *inside {
                    if *down {
                        self.mouse_down = true;
                        EventResponse::Sink
                    } else if self.mouse_down {
                        self.mouse_down = false;
                        self.just_toggled = true;
                        EventResponse::Sink
                    } else {
                        EventResponse::Bubble
                    }
                } else {
                    self.mouse_down = false;
                    EventResponse::Bubble
                }
            }
            WidgetEvent::FocusChanged(focused) => {
                self.focused = *focused;
                EventResponse::Bubble
            }
            WidgetEvent::KeyChanged {
                key: KeyCode::Enter | KeyCode::NumpadEnter,
                down,
                ..
            } => {
                if *down {
                    self.just_toggled = true;
                }
                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}
