use winit::event::{Event as WinitEvent, WindowEvent};
use yakui::{Rect, Vec2};

pub fn handle_event<T>(state: &mut yakui::State, event: &WinitEvent<T>) {
    #[allow(clippy::single_match)]
    match event {
        WinitEvent::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            let rect =
                Rect::from_pos_size(Vec2::ZERO, Vec2::new(size.width as f32, size.height as f32));

            state.handle_event(yakui::Event::SetViewport(rect));
        }
        WinitEvent::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            let pos = Vec2::new(position.x as f32, position.y as f32);
            state.handle_event(yakui::Event::MoveMouse(pos));
        }
        _ => (),
    }
}
