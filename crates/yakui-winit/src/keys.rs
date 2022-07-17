use winit::event::VirtualKeyCode;
use yakui_core::input::KeyboardKey;

pub fn from_winit_key(key: VirtualKeyCode) -> Option<KeyboardKey> {
    Some(match key {
        VirtualKeyCode::Space => KeyboardKey::Space,
        VirtualKeyCode::Back => KeyboardKey::Backspace,
        VirtualKeyCode::Delete => KeyboardKey::Delete,
        VirtualKeyCode::Return => KeyboardKey::Return,
        VirtualKeyCode::Tab => KeyboardKey::Tab,
        _ => return None,
    })
}
