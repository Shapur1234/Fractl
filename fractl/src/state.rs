use std::num::NonZeroU32;

use cgmath::Vector2;
use lib::{Camera, ColorType, Draw, Fill, Float, Fractal, FractalType, FrameBuffer, Label};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const DEFAULT_MAX_ITERATIONS: NonZeroU32 =
    unsafe { NonZeroU32::new_unchecked(if cfg!(debug_assertions) { 64 } else { 512 }) };
const DEFAULT_SHOW_CROSSHAIR: bool = true;
const DEFAULT_SHOW_UI: bool = true;

pub struct State {
    camera: Camera,
    selected_fractal_type: FractalType,
    selected_color_type: ColorType,
    max_iterations: NonZeroU32,
    show_crosshair: bool,
    show_ui: bool,
}

impl State {
    pub fn new(screen_size: impl Into<Vector2<NonZeroU32>>) -> Self {
        let screen_size = screen_size.into();
        Self {
            camera: Camera::new(screen_size),
            selected_fractal_type: FractalType::default(),
            selected_color_type: ColorType::default(),
            max_iterations: DEFAULT_MAX_ITERATIONS,
            show_crosshair: DEFAULT_SHOW_CROSSHAIR,
            show_ui: DEFAULT_SHOW_UI,
        }
    }

    pub fn resize(&mut self, new_screen_size: impl Into<Vector2<NonZeroU32>>) {
        self.camera.resize(new_screen_size);
    }

    pub fn render(&self, screen_size: impl Into<Vector2<NonZeroU32>>) -> Vec<u32> {
        let mut framebuffer = FrameBuffer::new(screen_size.into());

        let frametime = {
            let now = instant::Instant::now();

            Fractal::new(
                self.selected_fractal_type,
                self.selected_color_type,
                self.camera.clone(),
                self.max_iterations,
            )
            .fill(&mut framebuffer);

            now.elapsed()
        };

        if self.show_crosshair {
            const CROSSHAIR_SIZE: i64 = 5;

            let center = (framebuffer.size() / 2).map(|x| x as i64);
            for x in -CROSSHAIR_SIZE..=CROSSHAIR_SIZE {
                for y in -CROSSHAIR_SIZE..=CROSSHAIR_SIZE {
                    let current_pixel = Vector2::new((center.x + x) as u32, (center.y + y) as u32);

                    if x == 0 || y == 0 {
                        framebuffer[current_pixel] = framebuffer[current_pixel].invert();
                    }
                }
            }
        }

        if self.show_ui {
            let (start_y, line_offset) = (40, 40);
            Label::new("Fractaller", 40.0, None).draw(Vector2::new(10, start_y + line_offset / 2), &mut framebuffer);

            Label::new(format!("Selected fractal: {:}", self.selected_fractal_type), 25.0, None)
                .draw(Vector2::new(10, start_y + line_offset * 2), &mut framebuffer);
            Label::new(format!("Selected coloring: {:}", self.selected_color_type), 25.0, None)
                .draw(Vector2::new(10, start_y + line_offset * 3), &mut framebuffer);

            Label::new(format!("Max iterations: {:}", self.max_iterations), 25.0, None)
                .draw(Vector2::new(10, start_y + line_offset * 4), &mut framebuffer);

            Label::new(
                format!("Frametime: {:} ms", frametime.as_secs_f32() * 1000.0),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 5), &mut framebuffer);

            Label::new(
                format!(
                    "Center pos: ({:}, {:})",
                    self.camera.center_pos().x,
                    self.camera.center_pos().y
                ),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 6), &mut framebuffer);

            Label::new(
                format!(
                    "View size: ({:}, {:})",
                    self.camera.view_size().x,
                    self.camera.view_size().y
                ),
                25.0,
                None,
            )
            .draw(Vector2::new(10, start_y + line_offset * 7), &mut framebuffer);
        }

        framebuffer.raw()
    }

    fn handle_state_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        const CHANGE_MAX_ITERATIONS_MULT: Float = 1.5;
        const CHANGE_MULTIBROT_EXPONENT_STEP: Float = 0.05;

        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyK => {
                        self.max_iterations = NonZeroU32::new(
                            ((((self.max_iterations.get() as Float) * CHANGE_MAX_ITERATIONS_MULT).ceil()) as i64)
                                .try_into()
                                .unwrap_or_default(),
                        )
                        .unwrap_or(self.max_iterations);

                        true
                    }
                    KeyCode::KeyL => {
                        self.max_iterations = NonZeroU32::new(
                            ((((self.max_iterations.get() as Float) / CHANGE_MAX_ITERATIONS_MULT).ceil()) as i64)
                                .try_into()
                                .unwrap_or_default(),
                        )
                        .unwrap_or(self.max_iterations);
                        true
                    }
                    KeyCode::KeyN => {
                        self.selected_fractal_type = self.selected_fractal_type.prev();

                        true
                    }
                    KeyCode::KeyM => {
                        self.selected_fractal_type = self.selected_fractal_type.next();

                        true
                    }
                    KeyCode::KeyV => {
                        self.selected_color_type = self.selected_color_type.prev();

                        true
                    }
                    KeyCode::KeyB => {
                        self.selected_color_type = self.selected_color_type.next();

                        true
                    }
                    KeyCode::KeyC => {
                        self.show_crosshair ^= true;

                        true
                    }
                    KeyCode::KeyU => {
                        self.show_ui ^= true;

                        true
                    }
                    KeyCode::KeyZ => {
                        self.selected_fractal_type
                            .change_multi_parametr(-CHANGE_MULTIBROT_EXPONENT_STEP);

                        true
                    }
                    KeyCode::KeyX => {
                        self.selected_fractal_type
                            .change_multi_parametr(CHANGE_MULTIBROT_EXPONENT_STEP);

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

    pub fn handle_keyboard_input(&mut self, key_event: &KeyEvent) -> bool {
        self.handle_state_keyboard_input(key_event) || self.camera.handle_keyboard_input(key_event)
    }

    pub fn zoom_to(&mut self, by: f64, mouse_pos: Vector2<f64>, screen_size: Vector2<NonZeroU32>) {
        let mouse_world_pos = self
            .camera
            .screen_to_world_pos(&mouse_pos.map(|x| x as u32), &screen_size.map(|x| x.get()));

        self.camera.zoom_to(by as Float, mouse_world_pos);
    }
}
