mod keys;

use winit::dpi::PhysicalSize;
use winit::event::{
    DeviceEvent, ElementState, Event as WinitEvent, MouseButton as WinitMouseButton,
    MouseScrollDelta, WindowEvent,
};
use winit::window::Window;
use yakui_core::event::Event;
use yakui_core::geometry::{Rect, Vec2};
use yakui_core::input::MouseButton;

pub use self::keys::{from_winit_key, from_winit_modifiers};

#[non_exhaustive]
pub struct YakuiWinit {
    auto_scale: bool,
    auto_viewport: bool,
    init: Option<InitState>,
}

struct InitState {
    size: PhysicalSize<u32>,
    scale: f32,
}

impl YakuiWinit {
    #[allow(clippy::new_without_default)]
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let scale = window.scale_factor() as f32;

        Self {
            auto_scale: true,
            auto_viewport: true,
            init: Some(InitState { size, scale }),
        }
    }

    /// Configure whether scale factor (ie DPI) should be automatically applied
    /// from the window to scale the yakui UI.
    ///
    /// Defaults to `true`.
    pub fn set_automatic_scale_factor(&mut self, enabled: bool) {
        self.auto_scale = enabled;
    }

    /// Configure whether the viewport should be automatically updated to match
    /// the window size.
    ///
    /// Defaults to `true`.
    pub fn set_automatic_viewport(&mut self, enabled: bool) {
        self.auto_viewport = enabled;
    }

    pub fn handle_event<T>(
        &mut self,
        state: &mut yakui_core::Yakui,
        event: &WinitEvent<T>,
    ) -> bool {
        if let Some(init) = self.init.take() {
            let size = Vec2::new(init.size.width as f32, init.size.height as f32);
            state.set_surface_size(size);

            if self.auto_viewport {
                state.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
            }

            if self.auto_scale {
                state.set_scale_factor(init.scale);
            }
        }

        match event {
            WinitEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = Vec2::new(size.width as f32, size.height as f32);
                state.set_surface_size(size);

                if self.auto_viewport {
                    state.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
                }

                false
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                ..
            } => {
                if self.auto_scale {
                    state.set_scale_factor(*scale_factor as f32)
                }

                false
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let pos = Vec2::new(position.x as f32, position.y as f32);
                state.handle_event(Event::CursorMoved(Some(pos)))
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                ..
            } => state.handle_event(Event::CursorMoved(None)),
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
                    WinitMouseButton::Other(_) => return false,
                };

                let down = match button_state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                state.handle_event(Event::MouseButtonChanged { button, down })
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let delta = match *delta {
                    // Estimate how big a line is.
                    // TODO: Is there a better way to do this?
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y) * 16.0,
                    MouseScrollDelta::PixelDelta(offset) => {
                        Vec2::new(offset.x as f32, offset.y as f32)
                    }
                };

                state.handle_event(Event::MouseScroll { delta })
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => state.handle_event(Event::TextInput(*c)),
            WinitEvent::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => state.handle_event(Event::ModifiersChanged(from_winit_modifiers(*mods))),
            WinitEvent::DeviceEvent {
                event: DeviceEvent::Key(input),
                ..
            } => {
                if let Some(key) = input.virtual_keycode.and_then(from_winit_key) {
                    let pressed = match input.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    state.handle_event(Event::KeyChanged { key, down: pressed })
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
