mod camera;
mod framebuffer;
#[cfg(feature = "gpu")]
mod gpu;
mod math;
mod text;

pub use camera::Camera;
pub use framebuffer::{Color, Draw, Fill, FrameBuffer};
pub use math::{ColorType, Fractal, FractalType};
pub use text::Label;

#[cfg(all(feature = "multithread", feature = "gpu"))]
compile_error!("feature \"multithread\" and feature \"gpu\" cannot be enabled at the same time");

#[cfg(not(any(feature = "f32", feature = "f64")))]
compile_error!("feature \"f32\" or feature \"f64\" must be enabled");

#[cfg(all(feature = "f32", feature = "f64"))]
compile_error!("feature \"f32\" and feature \"f64\" cannot be enabled at the same time");

#[cfg(all(feature = "f64", feature = "gpu"))]
compile_error!("feature \"f64\" and feature \"gpu\" cannot be enabled at the same time");
