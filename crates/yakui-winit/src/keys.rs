use winit::event::VirtualKeyCode;
use yakui_core::input::KeyboardKey;

pub fn from_winit_key(key: VirtualKeyCode) -> Option<KeyboardKey> {
    Some(match key {
        VirtualKeyCode::Space => KeyboardKey::Space,
        VirtualKeyCode::Back => KeyboardKey::Backspace,
        VirtualKeyCode::Delete => KeyboardKey::Delete,
        VirtualKeyCode::Return => KeyboardKey::Return,
        VirtualKeyCode::Tab => KeyboardKey::Tab,
        VirtualKeyCode::Left => KeyboardKey::Left,
        VirtualKeyCode::Right => KeyboardKey::Right,
        VirtualKeyCode::Escape => KeyboardKey::Escape,
        VirtualKeyCode::Home => KeyboardKey::Home,
        VirtualKeyCode::End => KeyboardKey::End,
        VirtualKeyCode::Up => KeyboardKey::Up,
        VirtualKeyCode::Down => KeyboardKey::Down,
        VirtualKeyCode::PageUp => KeyboardKey::PageUp,
        VirtualKeyCode::PageDown => KeyboardKey::PageDown,
        _ => return None,
    })
}
