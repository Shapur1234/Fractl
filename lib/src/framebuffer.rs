use std::{
    num::NonZeroU32,
    ops::{Add, Deref, DerefMut, Index, IndexMut},
};

use cgmath::Vector2;
#[cfg(feature = "image")]
use image::{Rgb, RgbImage};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color(u32);

impl Color {
    #[allow(dead_code)]
    pub const WHITE: Self = Self::new(255, 255, 255);
    #[allow(dead_code)]
    pub const BLACK: Self = Self::new(0, 0, 0);
    #[allow(dead_code)]
    pub const RED: Self = Self::new(255, 0, 0);
    #[allow(dead_code)]
    pub const GREEN: Self = Self::new(0, 255, 0);
    #[allow(dead_code)]
    pub const BLUE: Self = Self::new(0, 0, 255);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self((blue as u32) | ((green as u32) << 8) | ((red as u32) << 16))
    }

    pub const fn red(&self) -> u8 {
        ((self.0 & 0xFF0000) >> 16) as u8
    }

    pub const fn green(&self) -> u8 {
        ((self.0 & 0xFF00) >> 8) as u8
    }

    pub const fn blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn scale(&self, times: f32) -> Self {
        assert!(times.is_finite() && (0.0..=1.0).contains(&times));

        Self::new(
            (self.red() as f32 * times) as u8,
            (self.green() as f32 * times) as u8,
            (self.blue() as f32 * times) as u8,
        )
    }

    pub const fn invert(&self) -> Self {
        Self::new(255 - self.red(), 255 - self.green(), 255 - self.blue())
    }
}

impl Deref for Color {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(255, 255, 255)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red().saturating_add(rhs.red()),
            self.green().saturating_add(rhs.green()),
            self.blue().saturating_add(rhs.blue()),
        )
    }
}

#[derive(Clone, Debug, Hash)]
pub struct FrameBuffer {
    pub(crate) data: Vec<Color>,
    size: Vector2<u32>,
}

impl FrameBuffer {
    pub fn new(size: impl Into<Vector2<NonZeroU32>>) -> Self {
        let size = size.into().map(|x| x.get());

        Self {
            data: vec![Color::default(); (size.x * size.y) as usize],
            size,
        }
    }

    pub fn map_pixels(&mut self, f: impl Fn(Vector2<u32>) -> Color + Send + Sync) {
        self.data = {
            let range = 0..self.size.x * self.size.y;

            range
                .into_iter()
                .map(|index| self.index_to_pos(index))
                .map(f)
                .collect::<Vec<_>>()
        };
    }

    pub fn raw(self) -> Vec<u32> {
        unsafe { transform_vec::<Color, u32>(self.data) }
    }

    #[cfg(feature = "image")]
    pub fn as_image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.size().x, self.size().y);

        for x in 0..self.size().x {
            for y in 0..self.size().y {
                let color = self[Vector2::new(x, y)];
                img.put_pixel(x, y, Rgb([color.red(), color.green(), color.blue()]));
            }
        }

        img
    }

    pub fn size(&self) -> &Vector2<u32> {
        &self.size
    }

    #[allow(dead_code)]
    pub fn pos_to_index(&self, buffer_pos: Vector2<u32>) -> u32 {
        buffer_pos.y * self.size.x + buffer_pos.x
    }

    #[allow(dead_code)]
    pub fn index_to_pos(&self, index: u32) -> Vector2<u32> {
        let x = index % self.size.x;
        let y = (index - x) / self.size.x;

        Vector2::new(x, y)
    }
}

impl Index<Vector2<u32>> for FrameBuffer {
    type Output = Color;

    fn index(&self, index: Vector2<u32>) -> &Self::Output {
        assert!((index.x < self.size.x) && (index.y < self.size.y));

        let index = self.pos_to_index(index) as usize;
        &self.data[index]
    }
}

impl IndexMut<Vector2<u32>> for FrameBuffer {
    fn index_mut(&mut self, index: Vector2<u32>) -> &mut Self::Output {
        assert!((index.x < self.size.x) && (index.y < self.size.y));

        let index = self.pos_to_index(index) as usize;
        &mut self.data[index]
    }
}

pub trait Draw {
    fn draw(&self, pos: Vector2<u32>, buffer: &mut FrameBuffer);
}

pub trait Fill {
    fn fill(&self, buffer: &mut FrameBuffer);
}

pub(crate) unsafe fn transform_vec<T, S>(mut v: Vec<T>) -> Vec<S> {
    // https://users.rust-lang.org/t/current-meta-converting-vec-u-vec-t-where/86603/5

    let len = v.len();
    let capacity = v.capacity();
    let ptr = v.as_mut_ptr().cast::<S>();

    std::mem::forget(v);

    Vec::from_raw_parts(ptr, len, capacity)
}
