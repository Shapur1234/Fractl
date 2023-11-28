use std::{f64::consts::PI, num::NonZeroU32};

use cgmath::Vector2;

use crate::framebuffer::Color;

#[allow(dead_code)]
pub fn mandelbrot_basic(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> Color {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

    if {
        let q = (world_pos.x - 0.25).powi(2) + world_pos.y.powi(2);
        q * (q + (world_pos.x - 0.25)) <= 0.25 * world_pos.y.powi(2)
    } {
        Color::BLUE
    } else {
        let max_iterations = max_iterations.get() as usize;

        let mut n = 0;
        let (mut x2, mut y2, mut x, mut y) = (0.0, 0.0, 0.0, 0.0);

        while x2 + y2 <= 4.0 && n < max_iterations {
            y = 2.0 * x * y + world_pos.y;
            x = x2 - y2 + world_pos.x;

            x2 = x.powi(2);
            y2 = y.powi(2);

            n += 1;
        }

        Color::new(0, 0, (((n as f64) / (max_iterations as f64)) * 255.0) as u8)
    }
}

#[allow(dead_code)]
pub fn mandelbrot_color(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> Color {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring

    fn calc_color(n: usize, max_iterations: usize) -> Color {
        let s = (n as f64) / (max_iterations as f64);
        let v = 1.0 - (PI * s).cos().powi(2);

        Color::new(
            (75.0 - (75.0 * v)) as u8,
            (28.0 + (75.0 - (75.0 * v))) as u8,
            ((360.0 * s).powf(1.5) % 360.0) as u8,
        )
    }

    let max_iterations = max_iterations.get() as usize;

    if {
        let q = (world_pos.x - 0.25).powi(2) + world_pos.y.powi(2);
        q * (q + (world_pos.x - 0.25)) <= 0.25 * world_pos.y.powi(2)
    } {
        calc_color(max_iterations, max_iterations)
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

        {
            calc_color(n, max_iterations)
        }
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
