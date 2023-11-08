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
        fn rgb_to_u32(red: u32, green: u32, blue: u32) -> u32 {
            blue | (green << 8) | (red << 16)
        }

        let screen_size = screen_size.into().map(|x| x.get());
        assert_eq!(buffer.len(), (screen_size.x * screen_size.y) as usize);

        let mut new_buffer = (0..buffer.len() as u32)
            .into_iter()
            .map(|index| {
                let screen_pos = index_to_pos(index, &screen_size);

                let (red, green, blue) = calculate_color(screen_pos, &self.camera);

                rgb_to_u32(red, green, blue)
            })
            .collect::<Vec<u32>>();

        // TODO: Optimize
        for i in 0..buffer.len() {
            std::mem::swap(&mut buffer[i], &mut new_buffer[i]);
        }
    }
}

fn calculate_color(screen_pos: Vector2<u32>, camera: &Camera) -> (u32, u32, u32) {
    let red = screen_pos.x % 255;
    let green = screen_pos.y % 255;
    let blue = (screen_pos.x * screen_pos.y) % 255;

    (red, green, blue)
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
