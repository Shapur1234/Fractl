mod state;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::{num::NonZeroU32, rc::Rc};

use cfg_if::cfg_if;
use cgmath::Vector2;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Fullscreen, WindowBuilder},
};

use crate::state::State;

#[cfg(all(feature = "multithread", feature = "gpu"))]
compile_error!("feature \"multithread\" and feature \"gpu\" cannot be enabled at the same time");

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(
        WindowBuilder::new()
            .with_title(env!("CARGO_BIN_NAME"))
            .with_inner_size(PhysicalSize::new(300, 200))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm::body().append_child(&window.canvas().unwrap()).unwrap();

        wasm::set_canvas_id();
        wasm::resize_canvas();
        wasm::register_window_resize();
    }

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    let mut mouse_pos = Vector2::new(0.0, 0.0);
    let mut screen_size = {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                 wasm::window_size()
            } else {
                let size = window.inner_size();
                Vector2::new(
                    NonZeroU32::new(size.width).unwrap_or(NonZeroU32::new(640).unwrap()),
                    NonZeroU32::new(size.height).unwrap_or(NonZeroU32::new(360).unwrap()),
                )
            }
        }
    };
    let mut state = State::new(screen_size);

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } if window_id == window.id() => {
                    if let (Some(screen_width), Some(screen_height)) = {
                        let size = window.inner_size();
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    } {
                        screen_size = Vector2::new(screen_width, screen_height);

                        surface.resize(screen_size.x, screen_size.y).unwrap();
                        state.resize(screen_size);

                        let mut buffer = surface.buffer_mut().unwrap();

                        let mut rendered_buffer = state.render(screen_size);
                        assert_eq!(rendered_buffer.len(), buffer.len());
                        for i in 0..buffer.len() {
                            std::mem::swap(&mut buffer[i], &mut rendered_buffer[i]);
                        }

                        buffer.present().unwrap();
                    }
                }
                Event::WindowEvent { window_id, event } => {
                    if window_id == window.id() {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        logical_key: Key::Named(NamedKey::Escape),
                                        ..
                                    },
                                ..
                            } => {
                                elwt.exit();
                            }
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        logical_key: Key::Named(NamedKey::F11),
                                        state: ElementState::Pressed,
                                        ..
                                    },
                                ..
                            } => {
                                if window.fullscreen().is_none() {
                                    window.set_fullscreen(Some(Fullscreen::Borderless(None)))
                                } else {
                                    window.set_fullscreen(None)
                                }
                            }
                            WindowEvent::KeyboardInput { event: key_event, .. } => {
                                if state.handle_keyboard_input(&key_event) {
                                    window.request_redraw()
                                }
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                mouse_pos.x = position.x;
                                mouse_pos.y = position.y;
                            }
                            WindowEvent::MouseWheel {
                                delta: MouseScrollDelta::LineDelta(_, y),
                                ..
                            } => {
                                state.zoom_to(y as f64, mouse_pos, screen_size);

                                window.request_redraw()
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
