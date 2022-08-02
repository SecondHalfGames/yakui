use winit::event::VirtualKeyCode;
use yakui_core::input::{KeyCode, Modifiers};

pub fn from_winit_key(key: VirtualKeyCode) -> Option<KeyCode> {
    Some(match key {
        VirtualKeyCode::Space => KeyCode::Space,
        VirtualKeyCode::Back => KeyCode::Backspace,
        VirtualKeyCode::Delete => KeyCode::Delete,
        VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => KeyCode::Enter,
        VirtualKeyCode::Tab => KeyCode::Tab,
        VirtualKeyCode::Left => KeyCode::ArrowLeft,
        VirtualKeyCode::Right => KeyCode::ArrowRight,
        VirtualKeyCode::Escape => KeyCode::Escape,
        VirtualKeyCode::Home => KeyCode::Home,
        VirtualKeyCode::End => KeyCode::End,
        VirtualKeyCode::Up => KeyCode::ArrowUp,
        VirtualKeyCode::Down => KeyCode::ArrowDown,
        VirtualKeyCode::PageUp => KeyCode::PageUp,
        VirtualKeyCode::PageDown => KeyCode::PageDown,

        VirtualKeyCode::A => KeyCode::KeyA,
        VirtualKeyCode::B => KeyCode::KeyB,
        VirtualKeyCode::C => KeyCode::KeyC,
        VirtualKeyCode::D => KeyCode::KeyD,
        VirtualKeyCode::E => KeyCode::KeyE,
        VirtualKeyCode::F => KeyCode::KeyF,
        VirtualKeyCode::G => KeyCode::KeyG,
        VirtualKeyCode::H => KeyCode::KeyH,
        VirtualKeyCode::I => KeyCode::KeyI,
        VirtualKeyCode::J => KeyCode::KeyJ,
        VirtualKeyCode::K => KeyCode::KeyK,
        VirtualKeyCode::L => KeyCode::KeyL,
        VirtualKeyCode::M => KeyCode::KeyM,
        VirtualKeyCode::N => KeyCode::KeyN,
        VirtualKeyCode::O => KeyCode::KeyO,
        VirtualKeyCode::P => KeyCode::KeyP,
        VirtualKeyCode::Q => KeyCode::KeyQ,
        VirtualKeyCode::R => KeyCode::KeyR,
        VirtualKeyCode::S => KeyCode::KeyS,
        VirtualKeyCode::T => KeyCode::KeyT,
        VirtualKeyCode::U => KeyCode::KeyU,
        VirtualKeyCode::V => KeyCode::KeyV,
        VirtualKeyCode::W => KeyCode::KeyW,
        VirtualKeyCode::X => KeyCode::KeyX,
        VirtualKeyCode::Y => KeyCode::KeyY,
        VirtualKeyCode::Z => KeyCode::KeyZ,
        VirtualKeyCode::Key0 => KeyCode::Digit0,
        VirtualKeyCode::Key1 => KeyCode::Digit1,
        VirtualKeyCode::Key2 => KeyCode::Digit2,
        VirtualKeyCode::Key3 => KeyCode::Digit3,
        VirtualKeyCode::Key4 => KeyCode::Digit4,
        VirtualKeyCode::Key5 => KeyCode::Digit5,
        VirtualKeyCode::Key6 => KeyCode::Digit6,
        VirtualKeyCode::Key7 => KeyCode::Digit7,
        VirtualKeyCode::Key8 => KeyCode::Digit8,
        VirtualKeyCode::Key9 => KeyCode::Digit9,
        VirtualKeyCode::F1 => KeyCode::F1,
        VirtualKeyCode::F2 => KeyCode::F2,
        VirtualKeyCode::F3 => KeyCode::F3,
        VirtualKeyCode::F4 => KeyCode::F4,
        VirtualKeyCode::F5 => KeyCode::F5,
        VirtualKeyCode::F6 => KeyCode::F6,
        VirtualKeyCode::F7 => KeyCode::F7,
        VirtualKeyCode::F8 => KeyCode::F8,
        VirtualKeyCode::F9 => KeyCode::F9,
        VirtualKeyCode::F10 => KeyCode::F10,
        VirtualKeyCode::F11 => KeyCode::F11,
        VirtualKeyCode::F12 => KeyCode::F12,
        VirtualKeyCode::F13 => KeyCode::F13,
        VirtualKeyCode::F14 => KeyCode::F14,
        VirtualKeyCode::F15 => KeyCode::F15,
        VirtualKeyCode::F16 => KeyCode::F16,
        VirtualKeyCode::F17 => KeyCode::F17,
        VirtualKeyCode::F18 => KeyCode::F18,
        VirtualKeyCode::F19 => KeyCode::F19,
        VirtualKeyCode::F20 => KeyCode::F20,
        VirtualKeyCode::F21 => KeyCode::F21,
        VirtualKeyCode::F22 => KeyCode::F22,
        VirtualKeyCode::F23 => KeyCode::F23,
        VirtualKeyCode::F24 => KeyCode::F24,

        _ => return None,
    })
}

pub fn from_winit_modifiers(winit_mods: winit::event::ModifiersState) -> Modifiers {
    let mut mods = Modifiers::default();
    if winit_mods.shift() {
        mods |= Modifiers::SHIFT;
    }
    if winit_mods.ctrl() {
        mods |= Modifiers::CONTROL;
    }
    if winit_mods.alt() {
        mods |= Modifiers::ALT;
    }
    if winit_mods.logo() {
        mods |= Modifiers::META;
    }
    mods
}
