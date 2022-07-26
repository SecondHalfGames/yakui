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
