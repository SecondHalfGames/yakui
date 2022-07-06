use std::collections::HashMap;

use glam::Vec2;

#[derive(Debug)]
pub struct InputState {
    pub mouse_position: Option<Vec2>,
    pub mouse_buttons: HashMap<MouseButton, ButtonState>,
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

    pub fn step(&mut self) {
        for state in self.mouse_buttons.values_mut() {
            state.settle();
        }
    }
}
