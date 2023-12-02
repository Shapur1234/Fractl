use std::num::NonZeroU32;

use cgmath::Vector2;
use lib::{Camera, Draw, Fractal, FractalType, FrameBuffer, Label};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

// TODO: Web

const DEFAULT_MAX_ITERATIONS: NonZeroU32 =
    unsafe { NonZeroU32::new_unchecked(if cfg!(debug_assertions) { 16 } else { 64 }) };

#[derive(Clone, Debug)]
pub struct State {
    camera: Camera,
    max_iterations: NonZeroU32,
}

impl State {
    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            camera: Camera::new(screen_size),
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.camera.resize(new_screen_size);
    }

    pub fn render(&self, screen_size: impl Into<Vector2<NonZeroU32>>) -> Vec<u32> {
        let mut framebuffer = FrameBuffer::new(screen_size.into());

        Fractal::new(FractalType::MandelbrotOLC, self.camera.clone(), self.max_iterations)
            .draw(Vector2::new(0, 0), &mut framebuffer);

        {
            let (start_y, line_offset) = (40, 40);
            Label::new("Fractaller", 30.0, None).draw(Vector2::new(10, start_y + line_offset * 0), &mut framebuffer);

            Label::new(format!("Max iterations: {:}", self.max_iterations), 25.0, None)
                .draw(Vector2::new(10, start_y + line_offset * 2), &mut framebuffer);

            Label::new(
                format!(
                    "Center pos: ({:}, {:})",
                    self.camera.center_pos().x,
                    self.camera.center_pos().y
                ),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 3), &mut framebuffer);

            Label::new(
                format!(
                    "View size: ({:}, {:})",
                    self.camera.view_size().x,
                    self.camera.view_size().y
                ),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 4), &mut framebuffer);
        }

        framebuffer.raw()
    }

    fn handle_state_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        const CHANGE_MAX_ITERATIONS_MULT: f64 = 1.5;

        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyK => {
                        self.max_iterations = NonZeroU32::new(
                            ((((self.max_iterations.get() as f64) * CHANGE_MAX_ITERATIONS_MULT).ceil()) as i64)
                                .try_into()
                                .unwrap_or_default(),
                        )
                        .unwrap_or(self.max_iterations);

                        true
                    }
                    KeyCode::KeyL => {
                        self.max_iterations = NonZeroU32::new(
                            ((((self.max_iterations.get() as f64) / CHANGE_MAX_ITERATIONS_MULT).ceil()) as i64)
                                .try_into()
                                .unwrap_or_default(),
                        )
                        .unwrap_or(self.max_iterations);
                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn handle_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        self.handle_state_keyboard_input(key_event) || self.camera.handle_keyboard_input(key_event)
    }

    pub fn zoom_to(&mut self, by: f64, mouse_pos: Vector2<f64>, screen_size: Vector2<NonZeroU32>) {
        let mouse_world_pos = self
            .camera
            .screen_to_world_pos(&mouse_pos.map(|x| x as u32), &screen_size.map(|x| x.get()));

        self.camera.zoom_to(by, mouse_world_pos);
    }
}
