use std::num::NonZeroU32;

use cgmath::Vector2;
use fontdue::Metrics;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{camera::Camera, math::mandelbrot};

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

    pub fn render(&self, buffer: &mut [u32], screen_size: impl Into<Vector2<NonZeroU32>>) {
        fn rgb_to_u32(red: u8, green: u8, blue: u8) -> u32 {
            (blue as u32) | ((green as u32) << 8) | ((red as u32) << 16)
        }

        let screen_size = screen_size.into().map(|x| x.get());
        assert_eq!(buffer.len(), (screen_size.x * screen_size.y) as usize);

        let range = 0..buffer.len() as u32;

        #[cfg(not(feature = "rayon"))]
        let iterator = range.into_iter();

        #[cfg(feature = "rayon")]
        let iterator = range.into_par_iter();

        let mut new_buffer = iterator
            .map(|index| {
                let screen_pos = index_to_pos(index, &screen_size);

                if screen_pos == screen_size / 2 {
                    return rgb_to_u32(255, 0, 0);
                }

                let (red, green, blue) = mandelbrot(
                    self.camera.screen_to_world_pos(&screen_pos, &screen_size),
                    self.max_iterations,
                );
                rgb_to_u32(red, green, blue)
            })
            .collect::<Vec<u32>>();

        for i in 0..buffer.len() {
            std::mem::swap(&mut buffer[i], &mut new_buffer[i]);
        }

        {
            let font = include_bytes!("../resource/JetBrainsMonoNerdFont-Regular.ttf") as &[u8];
            let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

            let (metrics, bitmap) = font.rasterize('j', 200.0);
            println!("{:?}, {:?}", metrics, bitmap.len());
            for x in 0..metrics.width as u32 {
                for y in 0..metrics.height as u32 {
                    buffer[pos_to_index(&Vector2::new(x, y), &screen_size) as usize] = rgb_to_u32(
                        bitmap[pos_to_index(
                            &Vector2::new(x, y),
                            &Vector2::new(metrics.width as u32, metrics.height as u32),
                        ) as usize],
                        0,
                        0,
                    );
                }
            }
        }
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
