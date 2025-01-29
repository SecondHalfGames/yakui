mod keys;

use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::video::Window;
use yakui_core::event::Event;
use yakui_core::geometry::{Rect, UVec2, Vec2};
use yakui_core::input::MouseButton;

use self::keys::{from_sdl_modifiers, from_sdl_scancode};

pub struct YakuiSdl2 {
    init: Option<InitState>,
}

struct InitState {
    size: UVec2,
    scale: f32,
}

fn scale_factor(window: &Window) -> f32 {
    let video = window.subsystem();
    let display_index = window.display_index().unwrap();
    video.display_dpi(display_index).unwrap().0 / 96.0
}

impl YakuiSdl2 {
    pub fn new(window: &Window) -> Self {
        let size = window.size().into();
        let scale = scale_factor(window);

        Self {
            init: Some(InitState { size, scale }),
        }
    }

    pub fn handle_event(&mut self, state: &mut yakui_core::Yakui, event: &SdlEvent) -> bool {
        if let Some(init) = self.init.take() {
            state.set_surface_size(init.size.as_vec2());
            state.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, init.size.as_vec2()));
            state.set_scale_factor(init.scale);
        }

        match event {
            SdlEvent::Window { win_event, .. } => {
                match win_event {
                    WindowEvent::Resized(x, y) => {
                        let size = Vec2::new(*x as f32, *y as f32);
                        state.set_surface_size(size);
                        state.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

                        false
                    }

                    // FIXME: scale factor changed
                    WindowEvent::Leave => state.handle_event(Event::CursorMoved(None)),

                    _ => false,
                }
            }

            SdlEvent::MouseMotion { x, y, .. } => {
                let pos = Vec2::new(*x as f32, *y as f32);
                state.handle_event(Event::CursorMoved(Some(pos)))
            }

            SdlEvent::MouseButtonDown { mouse_btn, .. } => {
                let button = match mouse_btn {
                    SdlMouseButton::Left => MouseButton::One,
                    SdlMouseButton::Right => MouseButton::Two,
                    SdlMouseButton::Middle => MouseButton::Three,
                    _ => return false,
                };

                state.handle_event(Event::MouseButtonChanged { button, down: true })
            }

            SdlEvent::MouseButtonUp { mouse_btn, .. } => {
                let button = match mouse_btn {
                    SdlMouseButton::Left => MouseButton::One,
                    SdlMouseButton::Right => MouseButton::Two,
                    SdlMouseButton::Middle => MouseButton::Three,
                    _ => return false,
                };

                state.handle_event(Event::MouseButtonChanged {
                    button,
                    down: false,
                })
            }

            SdlEvent::MouseWheel {
                precise_x,
                precise_y,
                ..
            } => {
                // Observed logical pixels per scroll wheel increment in Windows on Chrome
                const LINE_HEIGHT: f32 = 100.0 / 3.0;

                state.handle_event(Event::MouseScroll {
                    delta: Vec2::new(*precise_x, -*precise_y) * LINE_HEIGHT,
                })
            }

            SdlEvent::TextInput { text, .. } => {
                for c in text.chars() {
                    state.handle_event(Event::TextInput(c));
                }

                false
            }

            SdlEvent::KeyDown {
                scancode, keymod, ..
            } => {
                if let Some(key) = scancode.and_then(from_sdl_scancode) {
                    state.handle_event(Event::KeyChanged {
                        key,
                        down: true,
                        modifiers: Some(from_sdl_modifiers(*keymod)),
                    })
                } else {
                    false
                }
            }

            SdlEvent::KeyUp {
                scancode, keymod, ..
            } => {
                if let Some(key) = scancode.and_then(from_sdl_scancode) {
                    state.handle_event(Event::KeyChanged {
                        key,
                        down: false,
                        modifiers: Some(from_sdl_modifiers(*keymod)),
                    })
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}
