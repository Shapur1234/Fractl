use std::num::NonZeroU32;

use cgmath::Vector2;
use fractaller::{Camera, Draw, Fractal, FrameBuffer};
use image::{save_buffer_with_format, ColorType, ImageFormat};

const OUTPUT_SIZE: Vector2<NonZeroU32> = Vector2::new(unsafe { NonZeroU32::new_unchecked(3840) }, unsafe {
    NonZeroU32::new_unchecked(2160)
});
const MAX_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };
const ZOOM_LEVEL: Vector2<f64> = Vector2::new(0.4, 0.4);

fn main() {
    let camera = {
        let mut camera = Camera::new(OUTPUT_SIZE);
        camera.set_zoom(ZOOM_LEVEL);

        camera
    };

    let buffer = {
        let mut buffer = FrameBuffer::new(OUTPUT_SIZE);

        Fractal::new(fractaller::FractalType::MandelbrotOLC, camera, MAX_ITERATIONS)
            .draw(Vector2::new(0, 0), &mut buffer);
        buffer
    };

    save_buffer_with_format(
        "./out.png",
        &buffer.as_image(),
        buffer.size().x,
        buffer.size().y,
        ColorType::Rgb8,
        ImageFormat::Png,
    )
    .unwrap();
}
