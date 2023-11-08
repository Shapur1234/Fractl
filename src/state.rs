use std::num::NonZeroU32;

use cgmath::Vector2;

#[derive(Clone, Debug)]
pub struct State {
    camera: Camera,
}

impl State {
    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            camera: Camera::new(screen_size),
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.camera.resize(new_screen_size);
    }

    pub fn render(&self, buffer: &mut [u32], screen_size: impl Into<Vector2<NonZeroU32>>) {
        let screen_size = screen_size.into().map(|x| x.get());
        assert_eq!(buffer.len(), (screen_size.x * screen_size.y) as usize);

        for x in 0..screen_size.x {
            for y in 0..screen_size.y {
                let red = x % 255;
                let green = y % 255;
                let blue = (x * y) % 255;

                let index = y as usize * screen_size.x as usize + x as usize;
                buffer[index] = blue | (green << 8) | (red << 16);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Camera {
    center: Vector2<f64>,
    view_size: Vector2<f64>,
}

impl Camera {
    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            center: Vector2::new(0.0, 0.0),
            view_size: Vector2::new(Camera::calc_ratio(screen_size), 1.0),
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.view_size.x = Camera::calc_ratio(new_screen_size);
    }

    fn calc_ratio(new_screen_size: impl Into<Vector2<NonZeroU32>>) -> f64 {
        let new_screen_size = new_screen_size.into().map(|x| x.get() as f64);
        new_screen_size.x / new_screen_size.y
    }
}
