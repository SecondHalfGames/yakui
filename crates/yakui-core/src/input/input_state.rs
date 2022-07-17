use std::collections::HashMap;
use std::mem::take;

use glam::Vec2;
use smallvec::SmallVec;

use crate::dom::Dom;
use crate::event::{EventInterest, EventResponse, WidgetEvent};
use crate::id::WidgetId;
use crate::layout::LayoutDom;

use super::button::MouseButton;

#[derive(Debug)]
pub(crate) struct InputState {
    /// The current mouse position, or `None` if it's outside the window.
    pub mouse_position: Option<Vec2>,

    /// The state of each mouse button. If missing from the map, the button is
    /// up and has not yet been pressed.
    pub mouse_buttons: HashMap<MouseButton, ButtonState>,

    /// All of the widgets with mouse interest that the current mouse position
    /// intersects with.
    ///
    /// All lists like this are stored in reverse depth first order.
    pub mouse_hit: Vec<WidgetId>,

    /// All of the widgets that have had a mouse enter event sent to them
    /// without a corresponding mouse leave event yet. This is different from
    /// mouse_hit because hover events can be sunk by event handlers.
    pub mouse_entered: Vec<WidgetId>,

    /// All of the widgets that had a mouse enter event sent to them and then
    /// sunk it that are still being hovered. This helps us ensure that a widget
    /// that sunk a hover event will continue to occupy that space even if we
    /// don't send it more events.
    pub mouse_entered_and_sunk: Vec<WidgetId>,

    /// All widgets that had the corresponding mouse button pressed while the
    /// mouse cursor was over them.
    pub mouse_down_in: HashMap<MouseButton, Vec<WidgetId>>,

    // TODO: Remove this?
    pub mouse_hit_last: Vec<WidgetId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

impl ButtonState {
    pub fn is_down(&self) -> bool {
        matches!(self, Self::JustDown | Self::Down)
    }

    pub fn settle(&mut self) {
        match self {
            Self::JustDown => {
                *self = Self::Down;
            }
            Self::JustUp => {
                *self = Self::Up;
            }
            _ => (),
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_position: None,
            mouse_buttons: HashMap::new(),

            mouse_hit: Vec::new(),
            mouse_entered: Vec::new(),
            mouse_entered_and_sunk: Vec::new(),
            mouse_down_in: HashMap::new(),

            mouse_hit_last: Vec::new(),
        }
    }

    pub fn mouse_moved(&mut self, dom: &Dom, layout: &LayoutDom, pos: Option<Vec2>) {
        self.mouse_position = pos;
        self.mouse_hit_test(dom, layout);
        self.send_mouse_enter(dom);
        self.send_mouse_leave(dom);
    }

    pub fn mouse_button_changed(
        &mut self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        down: bool,
    ) -> EventResponse {
        let state = self.mouse_buttons.entry(button).or_insert(ButtonState::Up);

        match (state.is_down(), down) {
            // If the state didn't actually change, leave the current value
            // alone.
            (true, true) | (false, false) => EventResponse::Bubble,

            (false, true) => {
                *state = ButtonState::JustDown;
                self.send_button_change(dom, layout, button, true)
            }

            (true, false) => {
                *state = ButtonState::JustUp;
                self.send_button_change(dom, layout, button, false)
            }
        }
    }

    pub fn finish(&mut self) {
        self.settle_buttons();
    }

    fn send_button_change(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        value: bool,
    ) -> EventResponse {
        let mut overall_response = EventResponse::Bubble;

        for &id in &self.mouse_hit {
            if let Some(mut node) = dom.get_mut(id) {
                let response = node
                    .widget
                    .event(&WidgetEvent::MouseButtonChanged(button, value));

                if response == EventResponse::Sink {
                    overall_response = response;
                    break;
                }
            }
        }

        // For consistency, reverse the interest_mouse array like we do in
        // hit_test. This event can't be sunk, so it's not super important.
        let interest_mouse = layout.interest_mouse.iter().copied().rev();

        for (id, interest) in interest_mouse {
            if interest.contains(EventInterest::MOUSE_OUTSIDE) && !self.mouse_hit.contains(&id) {
                if let Some(mut node) = dom.get_mut(id) {
                    node.widget
                        .event(&WidgetEvent::MouseButtonChangedOutside(button, value));
                }
            }
        }

        overall_response
    }

    fn send_mouse_enter(&mut self, dom: &Dom) {
        for &hit in &self.mouse_hit {
            if let Some(mut node) = dom.get_mut(hit) {
                if !self.mouse_entered.contains(&hit) {
                    self.mouse_entered.push(hit);
                    let response = node.widget.event(&WidgetEvent::MouseEnter);

                    if response == EventResponse::Sink {
                        self.mouse_entered_and_sunk.push(hit);
                        break;
                    }
                } else if self.mouse_entered_and_sunk.contains(&hit) {
                    // This widget was hovered previously, is still hovered, and
                    // sunk the mouse enter event before. In order to prevent
                    // erroneously hovering other widgets, continue sinking this
                    // event.
                    break;
                }
            }
        }
    }

    fn send_mouse_leave(&mut self, dom: &Dom) {
        let mut to_remove = SmallVec::<[WidgetId; 4]>::new();

        for &hit in &self.mouse_entered {
            if !self.mouse_hit.contains(&hit) {
                if let Some(mut node) = dom.get_mut(hit) {
                    node.widget.event(&WidgetEvent::MouseLeave);
                }

                to_remove.push(hit);
            }
        }

        for remove in to_remove {
            self.mouse_entered.retain(|&id| id != remove);
            self.mouse_entered_and_sunk.retain(|&id| id != remove);
        }
    }

    fn mouse_hit_test(&mut self, dom: &Dom, layout: &LayoutDom) {
        let mut mouse_hit = take(&mut self.mouse_hit_last);
        mouse_hit.clear();
        self.mouse_hit_last = take(&mut self.mouse_hit);

        if let Some(mut mouse_pos) = self.mouse_position {
            mouse_pos /= layout.scale_factor();
            hit_test(dom, layout, mouse_pos, &mut mouse_hit);
        }
        self.mouse_hit = mouse_hit;
    }

    fn settle_buttons(&mut self) {
        for state in self.mouse_buttons.values_mut() {
            state.settle();
        }
    }
}

#[profiling::function]
fn hit_test(_dom: &Dom, layout: &LayoutDom, coords: Vec2, output: &mut Vec<WidgetId>) {
    // interest_mouse is stored in layout traversal order, which is depth first.
    //
    // We want to test against the deepest widgets in the tree first and bubble
    // to their ancestors first.
    let interest_mouse = layout.interest_mouse.iter().copied().rev();

    for (id, _interest) in interest_mouse {
        let layout_node = layout.get(id).unwrap();

        if layout_node.rect.contains_point(coords) {
            output.push(id);
        }
    }
}
