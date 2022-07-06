use glam::Vec2;

#[derive(Debug)]
pub struct InputState {
    pub mouse_position: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
        }
    }
}
