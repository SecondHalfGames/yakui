use winit::event::{
    ElementState, Event as WinitEvent, MouseButton as WinitMouseButton, WindowEvent,
};
use yakui_core::{Event, MouseButton, Rect, Vec2};

#[non_exhaustive]
pub struct State {}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    // TODO: How do we determine if an input event should be sunk by the UI?
    pub fn handle_event<T>(&mut self, state: &mut yakui_core::State, event: &WinitEvent<T>) {
        #[allow(clippy::single_match)]
        match event {
            WinitEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let rect = Rect::from_pos_size(
                    Vec2::ZERO,
                    Vec2::new(size.width as f32, size.height as f32),
                );

                state.handle_event(Event::SetViewport(rect));
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let pos = Vec2::new(position.x as f32, position.y as f32);
                state.handle_event(Event::MoveMouse(pos));
            }
            WinitEvent::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button,
                        state: button_state,
                        ..
                    },
                ..
            } => {
                let button = match button {
                    WinitMouseButton::Left => MouseButton::One,
                    WinitMouseButton::Right => MouseButton::Two,
                    WinitMouseButton::Middle => MouseButton::Three,
                    WinitMouseButton::Other(_) => return,
                };

                let down = match button_state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                state.handle_event(Event::MouseButtonChanged(button, down));
            }
            _ => (),
        }
    }
}
