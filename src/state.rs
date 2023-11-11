use std::num::NonZeroU32;

use cgmath::Vector2;
use num::{complex::ComplexFloat, Complex};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::Camera;

#[cfg(not(target_arch = "wasm32"))]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Debug)]
pub struct State {
    camera: Camera,
    max_iterations: NonZeroU32,
}

impl State {
    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            camera: Camera::new(screen_size),
            max_iterations: NonZeroU32::new(if cfg!(debug_assertions) { 10 } else { 50 }).unwrap(),
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.camera.resize(new_screen_size);
    }

    pub fn render(&self, buffer: &mut [u32], screen_size: impl Into<Vector2<NonZeroU32>>) {
        fn rgb_to_u32(red: u8, green: u8, blue: u8) -> u32 {
            (blue as u32) | ((green as u32) << 8) | ((red as u32) << 16)
        }

        let screen_size = screen_size.into().map(|x| x.get());
        assert_eq!(buffer.len(), (screen_size.x * screen_size.y) as usize);

        let range = 0..buffer.len() as u32;

        #[cfg(target_arch = "wasm32")]
        let iterator = range.into_iter();

        #[cfg(not(target_arch = "wasm32"))]
        let iterator = range.into_par_iter();

        let mut new_buffer = iterator
            .map(|index| {
                let screen_pos = index_to_pos(index, &screen_size);

                if screen_pos == screen_size / 2 {
                    return rgb_to_u32(255, 0, 0);
                }

                let (red, green, blue) = calculate_color(
                    self.camera.screen_to_world_pos(&screen_pos, &screen_size),
                    self.max_iterations,
                );
                rgb_to_u32(red, green, blue)
            })
            .collect::<Vec<u32>>();

        // TODO: Optimize
        for i in 0..buffer.len() {
            std::mem::swap(&mut buffer[i], &mut new_buffer[i]);
        }
    }

    fn handle_state_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyK => {
                        self.max_iterations = self.max_iterations.saturating_add(5);

                        true
                    }
                    KeyCode::KeyL => {
                        if let Some(sub_result) = self.max_iterations.get().checked_sub(5) {
                            if sub_result > 0 {
                                self.max_iterations = NonZeroU32::new(sub_result).unwrap();
                            }
                        }
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
}

fn calculate_color(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> (u8, u8, u8) {
    let max_iterations = max_iterations.get() as usize;

    let c = Complex::new(world_pos.x, world_pos.y);
    let (mut z, mut n): (Complex<f64>, usize) = (Complex::new(0.0, 0.0), 0);

    while z.abs() <= 2.0 && n < max_iterations {
        z = z.powi(2) + c;
        n += 1;
    }

    if z.abs() >= 2.0 {
        (
            ((n as f64 / max_iterations as f64) * 255.0) as u8,
            ((n as f64 / max_iterations as f64) * 255.0) as u8,
            ((n as f64 / max_iterations as f64) * 255.0) as u8,
        )
    } else {
        (0, 0, 0)
    }
}

#[allow(dead_code)]
fn pos_to_index(screen_pos: &Vector2<u32>, screen_size: &Vector2<u32>) -> u32 {
    screen_pos.y * screen_size.x + screen_pos.x
}

#[allow(dead_code)]
fn index_to_pos(index: u32, screen_size: &Vector2<u32>) -> Vector2<u32> {
    let x = index % screen_size.x;
    let y = (index - x) / screen_size.x;

    Vector2::new(x, y)
}
