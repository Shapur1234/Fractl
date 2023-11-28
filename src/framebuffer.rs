use std::{
    num::NonZeroU32,
    ops::{Add, Deref, DerefMut, Index, IndexMut},
};

use cgmath::Vector2;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

    #[allow(dead_code)]
    pub fn red(&self) -> u8 {
        ((self.0 & 0b00000000111111110000000000000000) >> 16) as u8
    }

    #[allow(dead_code)]
    pub fn green(&self) -> u8 {
        ((self.0 & 0b00000000000000001111111100000000) >> 8) as u8
    }

    #[allow(dead_code)]
    pub fn blue(&self) -> u8 {
        (self.0 & 0b00000000000000000000000011111111) as u8
    }

    #[allow(dead_code)]
    pub fn scale(&self, times: f32) -> Self {
        assert!(times.is_finite() && (0.0..=1.0).contains(&times));

        Self::new(
            (self.red() as f32 * times) as u8,
            (self.green() as f32 * times) as u8,
            (self.blue() as f32 * times) as u8,
        )
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
    data: Vec<Color>,
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

            #[cfg(not(feature = "rayon"))]
            let iterator = range.into_iter();

            #[cfg(feature = "rayon")]
            let iterator = range.into_par_iter();

            iterator
                .map(|index| self.index_to_pos(index))
                .map(f)
                .collect::<Vec<_>>()
        };
    }

    pub fn raw(self) -> Vec<u32> {
        /// SOURCE: https://users.rust-lang.org/t/current-meta-converting-vec-u-vec-t-where/86603/5
        ///
        /// Transmutes `Vec<T>` into `Vec<S>` in-place, without reallocation. The resulting
        /// vector has the same length and capacity.
        ///
        /// SAFETY: the types `T` and `S` must be transmute-compatible (same layout, and every
        /// representation of `T` must be a valid representation of some value in `S`).
        unsafe fn transform<T, S>(mut v: Vec<T>) -> Vec<S> {
            let len = v.len();
            let capacity = v.capacity();
            let ptr = v.as_mut_ptr().cast::<S>();
            // We must forget the original vector, otherwise it would deallocate the buffer on drop.
            std::mem::forget(v);
            // This is safe, because we are reusing a valid allocation of the same byte size.
            // The first `len` elements of `S` in this allocation must be initialized, which is
            // true since `size_of::<T>() == size_of::<S>()`, the first `len` elements of `T` are
            // initialized due the safety invariants of `Vec<T>`, and `T` and `S` being
            // transmute-compatible by the safety assumptions of this function.
            Vec::from_raw_parts(ptr, len, capacity)
        }

        unsafe { transform::<Color, u32>(self.data) }
    }

    pub fn size(&self) -> &Vector2<u32> {
        &self.size
    }

    #[allow(dead_code)]
    fn pos_to_index(&self, buffer_pos: Vector2<u32>) -> u32 {
        buffer_pos.y * self.size.x + buffer_pos.x
    }

    #[allow(dead_code)]
    fn index_to_pos(&self, index: u32) -> Vector2<u32> {
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
