use yakui_core::dom::Dom;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::colors;

const OUTER_SIZE: f32 = 32.0;
const INNER_SIZE: f32 = 24.0;

#[derive(Debug)]
#[non_exhaustive]
pub struct Checkbox {
    pub checked: bool,
}

impl Checkbox {
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }

    pub fn show(self) -> Response<CheckboxWidget> {
        crate::util::widget::<CheckboxWidget>(self)
    }
}

#[derive(Debug)]
pub struct CheckboxWidget {
    props: Checkbox,
    hovering: bool,
    mouse_down: bool,
    just_toggled: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct CheckboxResponse {
    pub checked: bool,
}

impl Widget for CheckboxWidget {
    type Props = Checkbox;
    type Response = CheckboxResponse;

    fn new(props: Self::Props) -> Self {
        Self {
            props,
            hovering: false,
            mouse_down: false,
            just_toggled: false,
        }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn respond(&mut self) -> Self::Response {
        let mut checked = self.props.checked;
        if self.just_toggled {
            checked = !checked;
            self.just_toggled = false;
        }

        CheckboxResponse { checked }
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let layout_node = layout.get(dom.current()).unwrap();
        let outer_rect = layout_node.rect.div_vec2(layout.viewport().size());

        let padding = Vec2::splat(OUTER_SIZE - INNER_SIZE);
        let mut check_rect = layout_node.rect;
        check_rect.set_pos(check_rect.pos() + padding / 2.0);
        check_rect.set_size(check_rect.size() - padding);
        check_rect = check_rect.div_vec2(layout.viewport().size());

        let mut bg = PaintRect::new(outer_rect);
        bg.color = colors::BACKGROUND_3;
        paint.add_rect(bg);

        if self.props.checked {
            crate::icons::cross(paint, check_rect, colors::TEXT);
        }
    }

    fn layout(&self, _dom: &Dom, _layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        constraints.constrain(Vec2::splat(OUTER_SIZE))
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
                EventResponse::Sink
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged(MouseButton::One, down) => {
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
            }
            WidgetEvent::MouseButtonChangedOutside(MouseButton::One, _) => {
                self.mouse_down = false;
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
