mod camera;
mod math;
mod state;

use std::{num::NonZeroU32, rc::Rc};

use cgmath::Vector2;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

use crate::state::State;

// TODO: Mouse controls
// TODO: UI - Egui
// TODO: Performance

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas().unwrap())
            .unwrap();
    }

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    let mut state = State::new({
        let size = window.inner_size();
        Vector2::new(
            NonZeroU32::new(size.width).unwrap_or(NonZeroU32::new(640).unwrap()),
            NonZeroU32::new(size.height).unwrap_or(NonZeroU32::new(360).unwrap()),
        )
    });

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } if window_id == window.id() => {
                    draw(&window, &mut surface, &mut state);
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        },
                    window_id,
                } if window_id == window.id() => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event: key_event, .. },
                    window_id,
                } => {
                    if window_id == window.id() && state.handle_keyboard_input(&key_event) {
                        window.request_redraw()
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

fn draw(
    window: &Rc<winit::window::Window>,
    surface: &mut softbuffer::Surface<Rc<winit::window::Window>, Rc<winit::window::Window>>,
    state: &mut State,
) {
    if let (Some(screen_width), Some(screen_height)) = {
        let size = window.inner_size();
        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    } {
        let screen_size = Vector2::new(screen_width, screen_height);

        {
            surface.resize(screen_size.x, screen_size.y).unwrap();
            state.resize(screen_size);
        }

        {
            let mut buffer = surface.buffer_mut().unwrap();

            state.render(&mut buffer, screen_size);

            buffer.present().unwrap();
        }
    }
}
