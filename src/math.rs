use std::num::NonZeroU32;

use cgmath::Vector2;

#[allow(dead_code)]
pub fn mandelbrot(world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> (u8, u8, u8) {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

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

    (
        ((n as f64 / max_iterations as f64) * 255.0) as u8 - 255,
        ((n as f64 / max_iterations as f64) * 255.0) as u8 - 255,
        ((n as f64 / max_iterations as f64) * 255.0) as u8 - 255,
    )
}

#[allow(dead_code)]
pub fn circle(world_pos: Vector2<f64>, _: NonZeroU32) -> (u8, u8, u8) {
    if world_pos.x.powi(2) + world_pos.y.powi(2) <= 1.0 {
        (255, 255, 255)
    } else {
        (0, 0, 0)
    }
}
