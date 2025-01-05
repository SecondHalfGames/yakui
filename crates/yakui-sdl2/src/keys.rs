use sdl2::keyboard::{Mod, Scancode};
use yakui_core::input::{KeyCode, Modifiers};

pub fn from_sdl_scancode(key: Scancode) -> Option<KeyCode> {
    Some(match key {
        Scancode::Space => KeyCode::Space,
        Scancode::Backspace => KeyCode::Backspace,
        Scancode::Delete => KeyCode::Delete,
        Scancode::Return => KeyCode::Enter,
        Scancode::KpEnter => KeyCode::NumpadEnter,
        Scancode::Tab => KeyCode::Tab,
        Scancode::Left => KeyCode::ArrowLeft,
        Scancode::Right => KeyCode::ArrowRight,
        Scancode::Escape => KeyCode::Escape,
        Scancode::Home => KeyCode::Home,
        Scancode::End => KeyCode::End,
        Scancode::Up => KeyCode::ArrowUp,
        Scancode::Down => KeyCode::ArrowDown,
        Scancode::PageUp => KeyCode::PageUp,
        Scancode::PageDown => KeyCode::PageDown,

        Scancode::A => KeyCode::KeyA,
        Scancode::B => KeyCode::KeyB,
        Scancode::C => KeyCode::KeyC,
        Scancode::D => KeyCode::KeyD,
        Scancode::E => KeyCode::KeyE,
        Scancode::F => KeyCode::KeyF,
        Scancode::G => KeyCode::KeyG,
        Scancode::H => KeyCode::KeyH,
        Scancode::I => KeyCode::KeyI,
        Scancode::J => KeyCode::KeyJ,
        Scancode::K => KeyCode::KeyK,
        Scancode::L => KeyCode::KeyL,
        Scancode::M => KeyCode::KeyM,
        Scancode::N => KeyCode::KeyN,
        Scancode::O => KeyCode::KeyO,
        Scancode::P => KeyCode::KeyP,
        Scancode::Q => KeyCode::KeyQ,
        Scancode::R => KeyCode::KeyR,
        Scancode::S => KeyCode::KeyS,
        Scancode::T => KeyCode::KeyT,
        Scancode::U => KeyCode::KeyU,
        Scancode::V => KeyCode::KeyV,
        Scancode::W => KeyCode::KeyW,
        Scancode::X => KeyCode::KeyX,
        Scancode::Y => KeyCode::KeyY,
        Scancode::Z => KeyCode::KeyZ,
        Scancode::Num0 => KeyCode::Digit0,
        Scancode::Num1 => KeyCode::Digit1,
        Scancode::Num2 => KeyCode::Digit2,
        Scancode::Num3 => KeyCode::Digit3,
        Scancode::Num4 => KeyCode::Digit4,
        Scancode::Num5 => KeyCode::Digit5,
        Scancode::Num6 => KeyCode::Digit6,
        Scancode::Num7 => KeyCode::Digit7,
        Scancode::Num8 => KeyCode::Digit8,
        Scancode::Num9 => KeyCode::Digit9,
        Scancode::F1 => KeyCode::F1,
        Scancode::F2 => KeyCode::F2,
        Scancode::F3 => KeyCode::F3,
        Scancode::F4 => KeyCode::F4,
        Scancode::F5 => KeyCode::F5,
        Scancode::F6 => KeyCode::F6,
        Scancode::F7 => KeyCode::F7,
        Scancode::F8 => KeyCode::F8,
        Scancode::F9 => KeyCode::F9,
        Scancode::F10 => KeyCode::F10,
        Scancode::F11 => KeyCode::F11,
        Scancode::F12 => KeyCode::F12,
        Scancode::F13 => KeyCode::F13,
        Scancode::F14 => KeyCode::F14,
        Scancode::F15 => KeyCode::F15,
        Scancode::F16 => KeyCode::F16,
        Scancode::F17 => KeyCode::F17,
        Scancode::F18 => KeyCode::F18,
        Scancode::F19 => KeyCode::F19,
        Scancode::F20 => KeyCode::F20,
        Scancode::F21 => KeyCode::F21,
        Scancode::F22 => KeyCode::F22,
        Scancode::F23 => KeyCode::F23,
        Scancode::F24 => KeyCode::F24,

        _ => return None,
    })
}

pub fn from_sdl_modifiers(sdl_mods: Mod) -> Modifiers {
    let mut mods = Modifiers::default();

    if sdl_mods.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        mods |= Modifiers::SHIFT;
    }

    if sdl_mods.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
        mods |= Modifiers::CONTROL;
    }

    if sdl_mods.intersects(Mod::LALTMOD) {
        mods |= Modifiers::ALT;
    }

    if sdl_mods.intersects(Mod::LGUIMOD | Mod::RGUIMOD) {
        mods |= Modifiers::META;
    }

    mods
}
