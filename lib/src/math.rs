use std::{f64::consts::PI, fmt::Display, num::NonZeroU32};

use cfg_if::cfg_if;
use cgmath::Vector2;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{framebuffer::Color, Camera, Draw};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FractalType {
    #[default]
    Mandelbrot,
    Multibrot,
}

impl FractalType {
    pub const fn next(&self) -> Self {
        match self {
            Self::Mandelbrot => Self::Multibrot,
            Self::Multibrot => Self::Mandelbrot,
        }
    }

    pub const fn prev(&self) -> Self {
        match self {
            Self::Mandelbrot => Self::Multibrot,
            Self::Multibrot => Self::Mandelbrot,
        }
    }

    pub fn escape_time(&self, world_pos: Vector2<f64>, max_iterations: NonZeroU32) -> u32 {
        match self {
            Self::Mandelbrot => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

                let max_iterations = max_iterations.get();

                let is_in_main_bulb = {
                    let q = (world_pos.x - 0.25).powi(2) + world_pos.y.powi(2);
                    q * (q + (world_pos.x - 0.25)) <= 0.25 * world_pos.y.powi(2)
                };

                if is_in_main_bulb {
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
            Self::Multibrot => todo!(),
        }
    }
}

impl Display for FractalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractalType::Mandelbrot => write!(f, "Mandelbrot"),
            FractalType::Multibrot => write!(f, "Multibrot"),
        }?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorType {
    #[default]
    Histogram,
    LCH,
    OLC,
}

impl ColorType {
    pub const fn next(&self) -> Self {
        match self {
            Self::Histogram => Self::LCH,
            Self::LCH => Self::OLC,
            Self::OLC => Self::Histogram,
        }
    }

    pub const fn prev(&self) -> Self {
        match self {
            Self::Histogram => Self::OLC,
            Self::LCH => Self::Histogram,
            Self::OLC => Self::LCH,
        }
    }

    pub fn escape_time_color(&self, escape_time: u32, max_iterations: NonZeroU32) -> Color {
        let max_iterations = max_iterations.get();
        match self {
            Self::Histogram => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring

                Color::new(0, 0, (((escape_time as f64) / (max_iterations as f64)) * 255.0) as u8)
            }
            Self::LCH => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring

                let s = (escape_time as f64) / (max_iterations as f64);
                let v = 1.0 - (PI * s).cos().powi(2);

                Color::new(
                    (75.0 - (75.0 * v)) as u8,
                    (28.0 + (75.0 - (75.0 * v))) as u8,
                    ((360.0 * s).powf(1.5) % 360.0) as u8,
                )
            }
            Self::OLC => {
                // https://github.com/OneLoneCoder/Javidx9/blob/54b26051d0fd1491c325ae09f50a7fc3f25030e8/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp#L543C3-L543C3

                let n = escape_time as f64;
                let a = 0.1;
                Color::new(
                    ((0.5 * (a * n).sin() + 0.5) * 255.0) as u8,
                    ((0.5 * (a * n + 2.094).sin() + 0.5) * 255.0) as u8,
                    ((0.5 * (a * n + 4.188).sin() + 0.5) * 255.0) as u8,
                )
            }
        }
    }
}

impl Display for ColorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorType::Histogram => write!(f, "Histogram"),
            ColorType::LCH => write!(f, "LCH"),
            ColorType::OLC => write!(f, "OLC"),
        }?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fractal {
    fractal_type: FractalType,
    color_type: ColorType,
    camera: Camera,
    max_iterations: NonZeroU32,
}

impl Fractal {
    pub fn new(kind: FractalType, color_type: ColorType, camera: Camera, max_iterations: NonZeroU32) -> Self {
        Self {
            fractal_type: kind,
            color_type,
            camera,
            max_iterations,
        }
    }
}

impl Draw for Fractal {
    fn draw(&self, pos: Vector2<u32>, buffer: &mut crate::FrameBuffer) {
        buffer.data = {
            let screen_poses = (0..buffer.size().x * buffer.size().y)
                .into_iter()
                .map(|index| buffer.index_to_pos(index))
                .collect::<Vec<_>>();

            let pixel_escape_times;
            cfg_if! {
                if #[cfg(feature = "rayon")] {
                    pixel_escape_times = screen_poses.into_par_iter().map(|screen_pos|
                        self.fractal_type.escape_time(
                            self.camera.screen_to_world_pos(&(screen_pos + pos), buffer.size()),
                            self.max_iterations,
                        )
                    ).collect::<Vec<_>>();
                } else {
                    pixel_escape_times = pixel_poses.into_iter().map(|pixel_pos|
                        self.fractal_type.escape_time(
                            self.camera.screen_to_world_pos(&(screen_pos + pos), buffer.size()),
                            self.max_iterations,
                        )
                    ).collect::<Vec<_>>();
                }
            };

            pixel_escape_times
                .into_iter()
                .map(|escape_time| self.color_type.escape_time_color(escape_time, self.max_iterations))
                .collect::<Vec<_>>()
        }
    }
}
