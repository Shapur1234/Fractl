use std::{fmt::Display, num::NonZeroU32};

use cfg_if::cfg_if;
use cgmath::Vector2;

use crate::{framebuffer::Color, Camera, Fill, Float};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FractalType {
    #[default]
    Mandelbrot,
    Multibrot(Float),
}

impl FractalType {
    const NUM_OF_VARIANTS: u8 = 2;
    const DEFAULT_MULTIBROT_ARGUEMENT: Float = 4.0;

    pub const fn id(&self) -> u8 {
        match self {
            Self::Mandelbrot => 0,
            Self::Multibrot(_) => 1,
        }
    }

    pub const fn from_id(id: u8) -> Self {
        match id % Self::NUM_OF_VARIANTS {
            0 => Self::Mandelbrot,
            1 => Self::Multibrot(Self::DEFAULT_MULTIBROT_ARGUEMENT),
            _ => unreachable!(),
        }
    }

    pub const fn next(&self) -> Self {
        Self::from_id(self.id() + 1)
    }

    pub const fn prev(&self) -> Self {
        Self::from_id(self.id() + Self::NUM_OF_VARIANTS - 1)
    }

    pub fn change_multi_parametr(&mut self, by: Float) {
        if let Self::Multibrot(exponent) = self {
            let new_exponent = *exponent + by;
            if new_exponent.is_finite() {
                *self = FractalType::Multibrot(new_exponent)
            }
        }
    }

    pub fn multi_parametr(&self) -> Option<Float> {
        match self {
            Self::Multibrot(exponent) => Some(*exponent),
            _ => None,
        }
    }

    pub fn escape_time(&self, world_pos: Vector2<Float>, max_iterations: NonZeroU32) -> u32 {
        let mut n = 0;
        let max_iterations = max_iterations.get();

        match self {
            Self::Mandelbrot => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

                let is_in_main_bulb = {
                    let q = (world_pos.x - 0.25).powi(2) + world_pos.y.powi(2);
                    q * (q + (world_pos.x - 0.25)) <= 0.25 * world_pos.y.powi(2)
                };

                if is_in_main_bulb {
                    max_iterations
                } else {
                    let (mut x2, mut y2, mut x, mut y) = (0.0, 0.0, 0.0, 0.0);

                    while (x2 + y2 <= 4.0) && (n < max_iterations) {
                        y = 2.0 * x * y + world_pos.y;
                        x = x2 - y2 + world_pos.x;

                        x2 = x.powi(2);
                        y2 = y.powi(2);

                        n += 1;
                    }

                    n
                }
            }
            Self::Multibrot(exponent) => {
                // https://en.wikipedia.org/wiki/Multibrot_set#Rendering_images

                let (mut x, mut y) = (world_pos.x, world_pos.y);

                while ((x.powi(2) + y.powi(2)) <= exponent.powi(2)) && (n < max_iterations) {
                    let x_y_squared_exp = (x.powi(2) + y.powi(2)).powf(exponent / 2.0);
                    let exponent_atan = exponent * y.atan2(x);

                    let x_tmp = x_y_squared_exp * (exponent_atan).cos() + world_pos.x;
                    y = x_y_squared_exp * (exponent_atan).sin() + world_pos.y;
                    x = x_tmp;

                    n += 1;
                }

                n
            }
        }
    }
}

impl Display for FractalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractalType::Mandelbrot => write!(f, "Mandelbrot"),
            FractalType::Multibrot(exponent) => write!(f, "Multibrot ({:?})", exponent),
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
    const NUM_OF_VARIANTS: u8 = 3;

    pub const fn id(&self) -> u8 {
        match self {
            Self::Histogram => 0,
            Self::LCH => 1,
            Self::OLC => 2,
        }
    }

    pub const fn from_id(id: u8) -> Self {
        match id % Self::NUM_OF_VARIANTS {
            0 => Self::Histogram,
            1 => Self::LCH,
            2 => Self::OLC,
            _ => unreachable!(),
        }
    }

    pub const fn next(&self) -> Self {
        Self::from_id(self.id() + 1)
    }

    pub const fn prev(&self) -> Self {
        Self::from_id(self.id() + Self::NUM_OF_VARIANTS - 1)
    }

    pub fn escape_time_color(&self, escape_time: u32, max_iterations: NonZeroU32) -> Color {
        let max_iterations = max_iterations.get();
        match self {
            Self::Histogram => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring

                Color::new(
                    0,
                    0,
                    (((escape_time as Float) / (max_iterations as Float)) * 255.0) as u8,
                )
            }
            Self::LCH => {
                // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring

                cfg_if! {
                    if #[cfg(feature = "f32")] {
                        const PI: f32 = std::f32::consts::PI;
                    } else if #[cfg(feature = "f64")] {
                        const PI: f64 = std::f64::consts::PI;
                    }
                }

                let s = (escape_time as Float) / (max_iterations as Float);
                let v = 1.0 - (PI * s).cos().powi(2);

                Color::new(
                    (75.0 - (75.0 * v)) as u8,
                    (28.0 + (75.0 - (75.0 * v))) as u8,
                    ((360.0 * s).powf(1.5) % 360.0) as u8,
                )
            }
            Self::OLC => {
                // https://github.com/OneLoneCoder/Javidx9/blob/54b26051d0fd1491c325ae09f50a7fc3f25030e8/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp#L543C3-L543C3

                let n = escape_time as Float;
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

impl Fill for Fractal {
    fn fill(&self, buffer: &mut crate::FrameBuffer) {
        buffer.data = {
            cfg_if! {
                if #[cfg(feature = "multithread")] {
                    use rayon::iter::{IntoParallelIterator, ParallelIterator};

                    (0..buffer.size().x.get() * buffer.size().y.get())
                        .into_par_iter()
                        .map(|index| buffer.index_to_pos(index))
                        .map(|screen_pos|
                            self.fractal_type.escape_time(
                                self.camera.screen_to_world_pos(&(screen_pos), buffer.size()),
                                self.max_iterations,
                            )
                        )
                        .map(|escape_time| self.color_type.escape_time_color(escape_time, self.max_iterations))
                        .collect::<Vec<_>>()
                } else if #[cfg(feature = "gpu")] {
                    use crate::{framebuffer::transform_vec, gpu::do_gpu_compute};


                    let mut io_buffer = (0..buffer.size().x.get() * buffer.size().y.get())
                        .into_iter().collect::<Vec<_>>();

                    do_gpu_compute(
                        &mut io_buffer,
                        &self.camera,
                        *buffer.size(),
                        self.max_iterations,
                        self.fractal_type,
                        self.color_type
                    );

                    unsafe { transform_vec::<u32, Color>(io_buffer) }
                } else {
                    (0..buffer.size().x.get() * buffer.size().y.get())
                        .map(|index| buffer.index_to_pos(index))
                        .map(|screen_pos|
                            self.fractal_type.escape_time(
                                self.camera.screen_to_world_pos(&(screen_pos), buffer.size()),
                                self.max_iterations,
                            )
                        )
                        .map(|escape_time| self.color_type.escape_time_color(escape_time, self.max_iterations))
                        .collect::<Vec<_>>()
                }
            }
        }
    }
}
