use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod ball;
mod border;
mod crates;
mod game;
mod physics;
mod platform;
mod rendering;

use game::*;

struct FpsLogger {
    last_log: std::time::Instant,
}

impl FpsLogger {
    fn new() -> Self {
        Self {
            last_log: std::time::Instant::now(),
        }
    }

    fn log(&mut self, now: std::time::Instant, dt: std::time::Duration) {
        if 1.0 <= (now - self.last_log).as_secs_f32() {
            println!(
                "Frame time: {:.2}ms(FPS: {:.2})",
                dt.as_secs_f64() * 1000.0,
                1.0 / dt.as_secs_f64()
            );
            self.last_log = now;
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut game = Game::new(&window);

    let mut last_render_time = std::time::Instant::now();
    let mut fps_logger = FpsLogger::new();
    _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key,
                            state,
                            ..
                        },
                    ..
                } => game.handle_input(key, state),
                WindowEvent::Resized(physical_size) => {
                    game.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    fps_logger.log(now, dt);

                    let dt = dt.as_secs_f32();

                    game.update(dt);
                    game.render_sync();
                    if !game.render() {
                        target.exit();
                    }
                }
                _ => {}
            },
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}
