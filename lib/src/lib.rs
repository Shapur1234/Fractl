mod camera;
mod framebuffer;
#[cfg(feature = "gpu")]
mod gpu;
mod math;
mod text;

pub use camera::Camera;
pub use framebuffer::{Color, Draw, FrameBuffer};
#[cfg(feature = "gpu")]
pub use gpu::{gpu_compute, WgpuContext};
pub use math::{ColorType, Fractal, FractalType};
pub use text::Label;

#[cfg(all(feature = "multithread", feature = "gpu"))]
compile_error!("feature \"multithread\" and feature \"gpu\" cannot be enabled at the same time");
