use std::{f64::consts::PI, num::NonZeroU32};

use cgmath::Vector2;

use crate::framebuffer::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FractalType {
    ManderbrolHistogram,
    MandelbrotLCH,
    MandelbrotOLC,
    Circle,
}

#[inline]
pub fn get_pixel(world_pos: Vector2<f64>, max_iterations: NonZeroU32, fractal_type: FractalType) -> Color {
    match fractal_type {
        FractalType::ManderbrolHistogram => {
            color_histogram(mandelbrot(world_pos, max_iterations), max_iterations.get())
        }
        FractalType::MandelbrotLCH => color_lhc(mandelbrot(world_pos, max_iterations), max_iterations.get()),
        FractalType::MandelbrotOLC => color_olc(mandelbrot(world_pos, max_iterations), max_iterations.get()),
        FractalType::Circle => circle(world_pos, max_iterations),
    }
}

#[allow(dead_code)]
fn mandelbrot(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> u32 {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

    let max_iterations = max_iterations.get();

    if {
        let q = (world_pos.x - 0.25).powi(2) + world_pos.y.powi(2);
        q * (q + (world_pos.x - 0.25)) <= 0.25 * world_pos.y.powi(2)
    } {
        max_iterations
    } else {
        let mut n = 0;
        let (mut x2, mut y2, mut x, mut y) = (0.0, 0.0, 0.0, 0.0);

        while x2 + y2 <= 4.0 && n < max_iterations {
            y = 2.0 * x * y + world_pos.y;
            x = x2 - y2 + world_pos.x;

            x2 = x.powi(2);
            y2 = y.powi(2);

            n += 1;
        }

        n
    }
}

fn color_histogram(n: u32, max_iterations: u32) -> Color {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring

    Color::new(0, 0, (((n as f64) / (max_iterations as f64)) * 255.0) as u8)
}

fn color_lhc(n: u32, max_iterations: u32) -> Color {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring

    let s = (n as f64) / (max_iterations as f64);
    let v = 1.0 - (PI * s).cos().powi(2);

    Color::new(
        (75.0 - (75.0 * v)) as u8,
        (28.0 + (75.0 - (75.0 * v))) as u8,
        ((360.0 * s).powf(1.5) % 360.0) as u8,
    )
}

fn color_olc(n: u32, _max_iterations: u32) -> Color {
    // https://github.com/OneLoneCoder/Javidx9/blob/54b26051d0fd1491c325ae09f50a7fc3f25030e8/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp#L543C3-L543C3

    let n = n as f64;
    let a = 0.1;
    Color::new(
        ((0.5 * (a * n).sin() + 0.5) * 255.0) as u8,
        ((0.5 * (a * n + 2.094).sin() + 0.5) * 255.0) as u8,
        ((0.5 * (a * n + 4.188).sin() + 0.5) * 255.0) as u8,
    )
}

#[allow(dead_code)]
fn circle(world_pos: Vector2<f64>, _: NonZeroU32) -> Color {
    if world_pos.x.powi(2) + world_pos.y.powi(2) <= 1.0 {
        Color::WHITE
    } else {
        Color::BLACK
    }
}
