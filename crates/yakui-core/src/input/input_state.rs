use std::collections::{HashMap, VecDeque};
use std::mem::take;

use glam::Vec2;
use thunderdome::Index;

use crate::dom::Dom;
use crate::event::EventInterest;
use crate::layout::LayoutDom;
use crate::WidgetEvent;

#[derive(Debug)]
pub struct InputState {
    pub mouse_position: Option<Vec2>,
    pub mouse_buttons: HashMap<MouseButton, ButtonState>,

    pub mouse_hit_last: Vec<Index>,
    pub mouse_hit: Vec<Index>,
    pub mouse_down_in: HashMap<MouseButton, Vec<Index>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    One,
    Two,
    Three,
}

impl ButtonState {
    pub fn is_down(&self) -> bool {
        matches!(self, Self::JustDown | Self::Down)
    }

    pub fn is_up(&self) -> bool {
        matches!(self, Self::JustUp | Self::Up)
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

            mouse_hit_last: Vec::new(),
            mouse_hit: Vec::new(),
            mouse_down_in: HashMap::new(),
        }
    }

    pub fn get_mouse_button(&self, button: MouseButton) -> ButtonState {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or(ButtonState::Up)
    }

    pub fn mouse_button_changed(&mut self, button: MouseButton, down: bool) {
        let state = self.mouse_buttons.entry(button).or_insert(ButtonState::Up);

        match (state.is_down(), down) {
            // If the state didn't actually change, leave the current value
            // alone.
            (true, true) | (false, false) => (),

            (true, false) => {
                *state = ButtonState::JustUp;
            }

            (false, true) => {
                *state = ButtonState::JustDown;
            }
        }
    }

    pub fn finish(&mut self, dom: &Dom, layout: &LayoutDom) {
        self.mouse_hit_test(dom, layout);
        self.send_mouse_events(dom);
        self.settle_buttons();
    }

    fn send_mouse_events(&self, dom: &Dom) {
        for &hit in &self.mouse_hit {
            if let Some(mut node) = dom.get_mut(hit) {
                if !self.mouse_hit_last.contains(&hit) {
                    node.widget.event(&WidgetEvent::MouseEnter);
                }

                for (&button, state) in self.mouse_buttons.iter() {
                    match state {
                        ButtonState::JustDown => node
                            .widget
                            .event(&WidgetEvent::MouseButtonChangedInside(button, true)),
                        ButtonState::JustUp => node
                            .widget
                            .event(&WidgetEvent::MouseButtonChangedInside(button, false)),
                        _ => (),
                    }
                }
            }
        }

        for &hit in &self.mouse_hit_last {
            if !self.mouse_hit.contains(&hit) {
                if let Some(mut node) = dom.get_mut(hit) {
                    node.widget.event(&WidgetEvent::MouseLeave);
                }
            }
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

fn hit_test(dom: &Dom, layout: &LayoutDom, coords: Vec2, output: &mut Vec<Index>) {
    let mut queue = VecDeque::new();

    queue.push_back(dom.root());

    while let Some(index) = queue.pop_front() {
        let node = dom.get(index).unwrap();
        let layout = layout.get(index).unwrap();

        if layout.rect.contains_point(coords) {
            if layout.event_interest.contains(EventInterest::MOUSE_INSIDE) {
                output.push(index);
            }

            queue.extend(&node.children);
        }
    }
}
