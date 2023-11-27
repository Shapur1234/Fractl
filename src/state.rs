use std::num::NonZeroU32;

use cgmath::Vector2;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    camera::Camera,
    framebuffer::{Color, Draw, FrameBuffer},
    math::mandelbrot,
    text::Label,
};

const DEFAULT_MAX_ITERATIONS: NonZeroU32 =
    unsafe { NonZeroU32::new_unchecked(if cfg!(debug_assertions) { 10 } else { 40 }) };

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
        let screen_size = *framebuffer.size();

        framebuffer.map_pixels(|screen_pos| {
            if screen_pos == screen_size / 2 {
                Color::RED
            } else {
                mandelbrot(
                    self.camera.screen_to_world_pos(&screen_pos, &screen_size),
                    self.max_iterations,
                )
            }
        });

        {
            let (start_y, line_offset) = (40, 40);
            Label::new("Fractaller", 30.0, None).draw(Vector2::new(10, start_y + line_offset * 0), &mut framebuffer);

            Label::new(format!("Max iterations: {:}", self.max_iterations), 25.0, None)
                .draw(Vector2::new(10, start_y + line_offset * 2), &mut framebuffer);

            Label::new(
                format!(
                    "Center pos: ({:.6}, {:.6})",
                    self.camera.center_pos().x,
                    self.camera.center_pos().y
                ),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 3), &mut framebuffer);

            Label::new(
                format!(
                    "View size: ({:.6}, {:.6})",
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
        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyK => {
                        self.max_iterations = self
                            .max_iterations
                            .saturating_mul(NonZeroU32::new(2).unwrap_or(self.max_iterations));

                        true
                    }
                    KeyCode::KeyL => {
                        self.max_iterations =
                            NonZeroU32::new(self.max_iterations.get() / 2).unwrap_or(self.max_iterations);
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
