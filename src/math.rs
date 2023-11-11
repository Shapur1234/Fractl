use std::num::NonZeroU32;

use cgmath::Vector2;
use num::{complex::ComplexFloat, Complex};

#[allow(dead_code)]
pub fn mandelbrot(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> (u8, u8, u8) {
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
pub fn circle(world_pos: Vector2<f64>, _: NonZeroU32) -> (u8, u8, u8) {
    if world_pos.x.powi(2) + world_pos.y.powi(2) <= 1.0 {
        (255, 255, 255)
    } else {
        (0, 0, 0)
    }
}
