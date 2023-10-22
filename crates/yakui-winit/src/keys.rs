use yakui_core::input::{KeyCode, Modifiers};

pub fn from_winit_key(key: winit::keyboard::KeyCode) -> Option<KeyCode> {
    use winit::keyboard::KeyCode as WinitKey;
    Some(match key {
        WinitKey::Space => KeyCode::Space,
        WinitKey::Backspace => KeyCode::Backspace,
        WinitKey::Delete => KeyCode::Delete,
        WinitKey::Enter => KeyCode::Enter,
        WinitKey::NumpadEnter => KeyCode::NumpadEnter,
        WinitKey::Tab => KeyCode::Tab,
        WinitKey::ArrowLeft => KeyCode::ArrowLeft,
        WinitKey::ArrowRight => KeyCode::ArrowRight,
        WinitKey::Escape => KeyCode::Escape,
        WinitKey::Home => KeyCode::Home,
        WinitKey::End => KeyCode::End,
        WinitKey::ArrowUp => KeyCode::ArrowUp,
        WinitKey::ArrowDown => KeyCode::ArrowDown,
        WinitKey::PageUp => KeyCode::PageUp,
        WinitKey::PageDown => KeyCode::PageDown,

        WinitKey::KeyA => KeyCode::KeyA,
        WinitKey::KeyB => KeyCode::KeyB,
        WinitKey::KeyC => KeyCode::KeyC,
        WinitKey::KeyD => KeyCode::KeyD,
        WinitKey::KeyE => KeyCode::KeyE,
        WinitKey::KeyF => KeyCode::KeyF,
        WinitKey::KeyG => KeyCode::KeyG,
        WinitKey::KeyH => KeyCode::KeyH,
        WinitKey::KeyI => KeyCode::KeyI,
        WinitKey::KeyJ => KeyCode::KeyJ,
        WinitKey::KeyK => KeyCode::KeyK,
        WinitKey::KeyL => KeyCode::KeyL,
        WinitKey::KeyM => KeyCode::KeyM,
        WinitKey::KeyN => KeyCode::KeyN,
        WinitKey::KeyO => KeyCode::KeyO,
        WinitKey::KeyP => KeyCode::KeyP,
        WinitKey::KeyQ => KeyCode::KeyQ,
        WinitKey::KeyR => KeyCode::KeyR,
        WinitKey::KeyS => KeyCode::KeyS,
        WinitKey::KeyT => KeyCode::KeyT,
        WinitKey::KeyU => KeyCode::KeyU,
        WinitKey::KeyV => KeyCode::KeyV,
        WinitKey::KeyW => KeyCode::KeyW,
        WinitKey::KeyX => KeyCode::KeyX,
        WinitKey::KeyY => KeyCode::KeyY,
        WinitKey::KeyZ => KeyCode::KeyZ,
        WinitKey::Digit0 => KeyCode::Digit0,
        WinitKey::Digit1 => KeyCode::Digit1,
        WinitKey::Digit2 => KeyCode::Digit2,
        WinitKey::Digit3 => KeyCode::Digit3,
        WinitKey::Digit4 => KeyCode::Digit4,
        WinitKey::Digit5 => KeyCode::Digit5,
        WinitKey::Digit6 => KeyCode::Digit6,
        WinitKey::Digit7 => KeyCode::Digit7,
        WinitKey::Digit8 => KeyCode::Digit8,
        WinitKey::Digit9 => KeyCode::Digit9,
        WinitKey::F1 => KeyCode::F1,
        WinitKey::F2 => KeyCode::F2,
        WinitKey::F3 => KeyCode::F3,
        WinitKey::F4 => KeyCode::F4,
        WinitKey::F5 => KeyCode::F5,
        WinitKey::F6 => KeyCode::F6,
        WinitKey::F7 => KeyCode::F7,
        WinitKey::F8 => KeyCode::F8,
        WinitKey::F9 => KeyCode::F9,
        WinitKey::F10 => KeyCode::F10,
        WinitKey::F11 => KeyCode::F11,
        WinitKey::F12 => KeyCode::F12,
        WinitKey::F13 => KeyCode::F13,
        WinitKey::F14 => KeyCode::F14,
        WinitKey::F15 => KeyCode::F15,
        WinitKey::F16 => KeyCode::F16,
        WinitKey::F17 => KeyCode::F17,
        WinitKey::F18 => KeyCode::F18,
        WinitKey::F19 => KeyCode::F19,
        WinitKey::F20 => KeyCode::F20,
        WinitKey::F21 => KeyCode::F21,
        WinitKey::F22 => KeyCode::F22,
        WinitKey::F23 => KeyCode::F23,
        WinitKey::F24 => KeyCode::F24,

        _ => return None,
    })
}

pub fn from_winit_modifiers(winit_mods: winit::keyboard::ModifiersState) -> Modifiers {
    let mut mods = Modifiers::default();
    if winit_mods.shift_key() {
        mods |= Modifiers::SHIFT;
    }
    if winit_mods.control_key() {
        mods |= Modifiers::CONTROL;
    }
    if winit_mods.alt_key() {
        mods |= Modifiers::ALT;
    }
    if winit_mods.super_key() {
        mods |= Modifiers::META;
    }
    mods
}
