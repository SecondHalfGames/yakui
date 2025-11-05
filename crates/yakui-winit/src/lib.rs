#![doc = include_str!("../README.md")]

mod keys;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::window::Window;
use yakui_core::event::Event;
use yakui_core::geometry::{Rect, Vec2};
use yakui_core::input::MouseButton;

pub use self::keys::{from_winit_key, from_winit_modifiers};

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

    pub fn handle_window_event(
        &mut self,
        state: &mut yakui_core::Yakui,
        event: &WindowEvent,
        window: &Window,
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

        if let Some(cursor) = state.get_text_cursor() {
            window.set_ime_cursor_area(
                PhysicalPosition::new(cursor.pos().x, cursor.pos().y),
                PhysicalSize::new(cursor.size().x, cursor.size().y),
            );
        }

        match event {
            WindowEvent::Resized(size) => {
                let size = Vec2::new(size.width as f32, size.height as f32);
                state.set_surface_size(size);

                if self.auto_viewport {
                    state.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
                }

                false
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if self.auto_scale {
                    state.set_scale_factor(*scale_factor as f32)
                }

                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Vec2::new(position.x as f32, position.y as f32);
                state.handle_event(Event::CursorMoved(Some(pos)))
            }
            WindowEvent::CursorLeft { .. } => state.handle_event(Event::CursorMoved(None)),

            WindowEvent::MouseInput {
                button,
                state: button_state,
                ..
            } => {
                let button = match button {
                    WinitMouseButton::Left => MouseButton::One,
                    WinitMouseButton::Right => MouseButton::Two,
                    WinitMouseButton::Middle => MouseButton::Three,
                    _ => return false,
                };

                let down = match button_state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                state.handle_event(Event::MouseButtonChanged { button, down })
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Observed logical pixels per scroll wheel increment in Windows on Chrome
                const LINE_HEIGHT: f32 = 100.0 / 3.0;

                let delta = match *delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y) * LINE_HEIGHT,
                    MouseScrollDelta::PixelDelta(offset) => {
                        Vec2::new(offset.x as f32, offset.y as f32)
                            / state.layout_dom().scale_factor()
                    }
                };

                // Flip delta axis from winit's expectations.
                let delta = -delta;

                state.handle_event(Event::MouseScroll { delta })
            }
            WindowEvent::ModifiersChanged(mods) => {
                state.handle_event(Event::ModifiersChanged(from_winit_modifiers(mods.state())))
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(text) = event.text.as_ref() {
                        for c in text.chars() {
                            state.handle_event(Event::TextInput(c));
                        }
                    }
                }
                let key = match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(k) => from_winit_key(k),
                    winit::keyboard::PhysicalKey::Unidentified(_) => None,
                };
                if let Some(key) = key {
                    let pressed = match event.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    state.handle_event(Event::KeyChanged {
                        key,
                        down: pressed,
                        modifiers: None,
                    })
                } else {
                    false
                }
            }

            WindowEvent::Ime(winit::event::Ime::Preedit(text, position)) => {
                state.handle_event(Event::TextPreedit(text.clone(), *position));
                true
            }

            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                for c in text.chars() {
                    state.handle_event(Event::TextInput(c));
                }
                true
            }
            _ => false,
        }
    }
}
