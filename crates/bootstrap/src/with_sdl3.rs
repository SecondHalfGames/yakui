use std::time::Instant;

use sdl3::event::{Event, WindowEvent};
use yakui::{UVec2, Yakui};
use yakui_sdl3::YakuiSdl3;

use crate::graphics::Graphics;
use crate::{get_sample_count, ExampleBody, ExampleState};

pub fn run<T: ExampleBody>(mut yak: Yakui, mut state: ExampleState, title: String, body: T) {
    let start = Instant::now();

    sdl3::hint::set("SDL_MOUSE_RELATIVE_SCALING", "0");
    sdl3::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut window_builder = video_subsystem.window(&title, 800, 600);

    window_builder.resizable().position_centered().metal_view();

    let window = window_builder.build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let (width, height) = window.size();
    let size = UVec2::new(width, height);

    let mut yak_window = YakuiSdl3::new(&window);
    let mut graphics = pollster::block_on(Graphics::new(&window, size, get_sample_count()));

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            if yak_window.handle_event(&mut yak, &event) {
                continue;
            }

            match event {
                Event::Quit { .. } => {
                    break 'main_loop;
                }

                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => graphics.resize(UVec2::new(x as u32, y as u32)),
                _ => {}
            }
        }

        state.time = (Instant::now() - start).as_secs_f32();

        {
            profiling::scope!("Build UI");

            yak.start();
            body.run(&mut state);
            yak.finish();
        }

        yak_window.update(&window, &mut yak);

        graphics.paint(&mut yak, {
            let bg = yakui::colors::BACKGROUND_1.to_linear();
            wgpu::Color {
                r: bg.x.into(),
                g: bg.y.into(),
                b: bg.z.into(),
                a: 1.0,
            }
        });

        profiling::finish_frame!();
    }
}
