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
        fn set_pixel(
            screen_pos: &Vector2<u32>,
            screen_size: &Vector2<u32>,
            red: u32,
            green: u32,
            blue: u32,
            buffer: &mut [u32],
        ) {
            let index = screen_pos.y as usize * screen_size.x as usize + screen_pos.x as usize;
            buffer[index] = blue | (green << 8) | (red << 16);
        }

        let screen_size = screen_size.into().map(|x| x.get());
        assert_eq!(buffer.len(), (screen_size.x * screen_size.y) as usize);

        PixelIterator::new(screen_size).for_each(|screen_pos| {
            let red = screen_pos.x % 255;
            let green = screen_pos.y % 255;
            let blue = (screen_pos.x * screen_pos.y) % 255;

            set_pixel(&screen_pos, &screen_size, red, green, blue, buffer);
        });
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

struct PixelIterator {
    screen_size: Vector2<u32>,
    x: u32,
    y: u32,
}

impl PixelIterator {
    pub fn new(screen_size: impl Into<Vector2<u32>>) -> Self {
        Self {
            screen_size: screen_size.into(),
            x: 0,
            y: 0,
        }
    }
}

impl Iterator for PixelIterator {
    type Item = Vector2<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let out = Vector2::new(self.x, self.y);

        self.x += 1;
        if self.x >= self.screen_size.x {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= self.screen_size.y {
            None
        } else {
            Some(out)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let estimate = self.screen_size.x as usize * self.screen_size.y as usize;
        (estimate, Some(estimate))
    }
}
