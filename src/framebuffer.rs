use std::{
    num::NonZeroU32,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use cgmath::Vector2;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color(u32);

impl Color {
    #[allow(dead_code)]
    pub const RED: Self = Self::new(255, 0, 0);
    #[allow(dead_code)]
    pub const GREEN: Self = Self::new(0, 255, 0);
    #[allow(dead_code)]
    pub const BLUE: Self = Self::new(0, 0, 255);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self((blue as u32) | ((green as u32) << 8) | ((red as u32) << 16))
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

    pub fn to_raw(self) -> Vec<u32> {
        self.data.into_iter().map(|color| *color).collect()
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
