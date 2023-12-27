use std::num::NonZeroU32;

use cgmath::Vector2;
#[cfg(feature = "winit")]
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::Float;

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub(crate) center_pos: Vector2<Float>,
    pub(crate) view_size: Vector2<Float>,
    pub(crate) zoom: Vector2<Float>,
}

impl Camera {
    #[allow(dead_code)]
    const MOVE_INCREMENT: Float = 0.005;
    #[allow(dead_code)]
    const ZOOM_INCREMENT: Float = 0.02;
    #[allow(dead_code)]
    const MIN_ZOOM: Float = 0.1;
    #[allow(dead_code)]
    const MAX_ZOOM: Float = Float::MAX;

    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        Self {
            center_pos: Vector2::new(0.0, 0.0),
            view_size: Vector2::new(Camera::calc_ratio(screen_size), 1.0),
            zoom: Vector2::new(1.0, 1.0),
        }
    }

    #[allow(dead_code)]
    pub fn center_pos(&self) -> Vector2<Float> {
        self.center_pos
    }

    #[allow(dead_code)]
    pub fn view_size(&self) -> Vector2<Float> {
        self.view_size.zip(self.zoom, |x, y| x / y)
    }

    #[allow(dead_code)]
    pub fn zoom(&self) -> Vector2<Float> {
        self.zoom
    }

    #[allow(dead_code)]
    pub fn set_zoom(&mut self, new_zoom: Vector2<Float>) {
        assert!(new_zoom.x.is_finite() && new_zoom.y.is_finite());
        self.zoom = new_zoom;
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.view_size.x = Camera::calc_ratio(new_screen_size);
    }

    fn calc_ratio(new_screen_size: impl Into<Vector2<NonZeroU32>>) -> Float {
        let new_screen_size = new_screen_size.into().map(|x| x.get() as Float);
        new_screen_size.x / new_screen_size.y
    }

    #[cfg(feature = "winit")]
    pub fn handle_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyW => {
                        self.center_pos.y -= (Camera::MOVE_INCREMENT) / self.zoom.y;

                        true
                    }
                    KeyCode::KeyS => {
                        self.center_pos.y += (Camera::MOVE_INCREMENT) / self.zoom.y;

                        true
                    }
                    KeyCode::KeyA => {
                        self.center_pos.x -= (Camera::MOVE_INCREMENT) / self.zoom.x;

                        true
                    }
                    KeyCode::KeyD => {
                        self.center_pos.x += (Camera::MOVE_INCREMENT) / self.zoom.x;

                        true
                    }
                    KeyCode::ArrowUp => {
                        self.zoom.y += Camera::ZOOM_INCREMENT * self.zoom.y;
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowDown => {
                        self.zoom.y -= Camera::ZOOM_INCREMENT * self.zoom.y;
                        self.zoom.y = self.zoom.y.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowRight => {
                        self.zoom.x += Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::ArrowLeft => {
                        self.zoom.x -= Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.x = self.zoom.x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);

                        true
                    }
                    KeyCode::KeyO => {
                        self.zoom.x += Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.y += Camera::ZOOM_INCREMENT * self.zoom.y;

                        self.zoom = self.zoom.map(|x| x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM));

                        true
                    }
                    KeyCode::KeyP => {
                        self.zoom.x -= Camera::ZOOM_INCREMENT * self.zoom.x;
                        self.zoom.y -= Camera::ZOOM_INCREMENT * self.zoom.y;

                        self.zoom = self.zoom.map(|x| x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM));

                        true
                    }
                    KeyCode::KeyI => {
                        self.zoom = Vector2::new(1.0, 1.0);

                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn zoom_to(&mut self, by: Float, world_pos: Vector2<Float>) {
        let zoom_old = self.zoom;

        self.zoom.x += Camera::ZOOM_INCREMENT * self.zoom.x * by;
        self.zoom.y += Camera::ZOOM_INCREMENT * self.zoom.y * by;

        self.zoom = self.zoom.map(|x| x.clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM));

        let (world_pos_delta, zoom_delta) = (world_pos - self.center_pos, self.zoom - zoom_old);
        let world_pos_delta_normed = world_pos_delta.map(|x| {
            if x.is_sign_positive() {
                x.sqrt()
            } else {
                -x.abs().sqrt()
            }
        });
        if world_pos_delta_normed.x.is_normal()
            && world_pos_delta_normed.y.is_normal()
            && zoom_delta.x.is_normal()
            && zoom_delta.y.is_normal()
        {
            self.center_pos += world_pos_delta_normed
                .zip(zoom_delta, |x, y| x * y)
                .zip(self.zoom, |x, y| x / y);
        }
    }

    pub fn screen_to_world_pos(&self, screen_pos: &Vector2<u32>, screen_size: &Vector2<u32>) -> Vector2<Float> {
        let screen_pos_normalized = screen_pos.zip(*screen_size, |pos, size| (pos as Float / size as Float) - 0.5);

        Vector2::new(
            ((screen_pos_normalized.x * self.view_size.x) / self.zoom.x) + self.center_pos.x,
            ((screen_pos_normalized.y * self.view_size.y) / self.zoom.y) + self.center_pos.y,
        )
    }
}
