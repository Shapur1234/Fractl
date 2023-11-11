use std::num::NonZeroU32;

use cgmath::Vector2;
use num::{complex::ComplexFloat, Complex};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

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
                let (red, green, blue) =
                    calculate_color(self.camera.world_pos(&screen_pos, &screen_size), self.max_iterations);
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

#[derive(Clone, Debug, PartialEq)]
struct Camera {
    center_pos: Vector2<f64>,
    view_size: Vector2<f64>,
    zoom: Vector2<f64>,
}

impl Camera {
    const MOVE_INCREMENT: f64 = 0.005;
    const ZOOM_INCREMENT: f64 = 0.02;
    const MIN_ZOOM: f64 = 0.1;
    const MAX_ZOOM: f64 = f64::MAX;

    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            center_pos: Vector2::new(0.0, 0.0),
            view_size: Vector2::new(Camera::calc_ratio(screen_size), 1.0),
            zoom: Vector2::new(1.0, 1.0),
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.view_size.x = Camera::calc_ratio(new_screen_size);
    }

    fn calc_ratio(new_screen_size: impl Into<Vector2<NonZeroU32>>) -> f64 {
        let new_screen_size = new_screen_size.into().map(|x| x.get() as f64);
        new_screen_size.x / new_screen_size.y
    }

    pub fn handle_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyW => {
                        self.center_pos.y -= (Camera::MOVE_INCREMENT) / self.zoom.y;

                        true
                    }
                    KeyCode::KeyS => {
                        self.center_pos.y += (Camera::MOVE_INCREMENT) / self.zoom.y;

                        true
                    }
                    KeyCode::KeyA => {
                        self.center_pos.x -= (Camera::MOVE_INCREMENT) / self.zoom.x;

                        true
                    }
                    KeyCode::KeyD => {
                        self.center_pos.x += (Camera::MOVE_INCREMENT) / self.zoom.x;

                        true
                    }
                    KeyCode::ArrowUp => {
                        self.zoom.y += Camera::ZOOM_INCREMENT * self.zoom.y;
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowDown => {
                        self.zoom.y -= Camera::ZOOM_INCREMENT * self.zoom.y;
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowRight => {
                        self.zoom.x += Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowLeft => {
                        self.zoom.x -= Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::KeyO => {
                        self.zoom.x += Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.y += Camera::ZOOM_INCREMENT * self.zoom.y;

                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::KeyP => {
                        self.zoom.x -= Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.y -= Camera::ZOOM_INCREMENT * self.zoom.y;

                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::KeyI => {
                        self.zoom = Vector2::new(1.0, 1.0);

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

    fn world_pos(&self, screen_pos: &Vector2<u32>, screen_size: &Vector2<u32>) -> Vector2<f64> {
        let (screen_pos, screen_size) = (screen_pos.map(|x| x as f64), screen_size.map(|x| x as f64));

        Vector2::new(
            ((screen_pos.x * self.view_size.x / self.zoom.x) / screen_size.x) + self.center_pos.x
                - self.view_size.x / 2.0,
            ((screen_pos.y * self.view_size.y / self.zoom.y) / screen_size.y) + self.center_pos.y
                - self.view_size.y / 2.0,
        )
    }
}

#[allow(dead_code)]
fn pos_to_index(screen_pos: &Vector2<u32>, screen_size: &Vector2<u32>) -> u32 {
    screen_pos.y * screen_size.x + screen_pos.x
}

#[allow(dead_code)]
fn index_to_pos(index: u32, screen_size: &Vector2<u32>) -> Vector2<u32> {
    let x = index % screen_size.x;
    let y = (index - x) / screen_size.y;

    Vector2::new(x, y)
}
