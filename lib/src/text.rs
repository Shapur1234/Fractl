use cgmath::Vector2;
use fontdue::{Font, FontSettings};
use lazy_static::lazy_static;

use crate::framebuffer::{Color, Draw, FrameBuffer};

pub struct Label {
    text: String,
    color: Color,
    fontsize: f32,
}

static FONT_BYTES: &[u8] = include_bytes!("../resource/WantedSans-Regular.ttf");
lazy_static! {
    static ref FONT: Font = Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap();
}

impl Label {
    #[allow(clippy::missing_errors_doc, clippy::needless_pass_by_value)]
    pub fn new(text: impl ToString, fontsize: f32, color: Option<Color>) -> Result<Self, &'static str> {
        let text = text.to_string();

        if !(fontsize.is_normal() && fontsize.is_sign_positive()) {
            Err("size must be normal and positive")
        } else if text.is_empty() {
            Err("text cannot be empty string")
        } else {
            Ok(Self {
                text,
                color: color.unwrap_or_default(),
                fontsize,
            })
        }
    }
}

impl Draw for Label {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn draw(&self, pos: Vector2<u32>, buffer: &mut FrameBuffer) {
        const SPACE_BETWEEN_CHARS_MULT: f32 = 10.0;
        const SIZE_OF_SPACE_MULT: f32 = 2.0;

        let mut x_offset = 0;
        for char in self.text.chars() {
            if char.is_whitespace() {
                x_offset += (self.fontsize / SIZE_OF_SPACE_MULT) as u32;
            } else {
                let (glyph_size, ymin, glyph_bitmap) = {
                    let (metrics, bitmap) = FONT.rasterize(char, self.fontsize);

                    (
                        Vector2::new(metrics.width as u32, metrics.height as u32),
                        i64::from(metrics.ymin),
                        bitmap,
                    )
                };

                for x in 0..glyph_size.x {
                    for y in 0..glyph_size.y {
                        let glyph_pos = Vector2::new(x, y);
                        let in_buffer_pos = Vector2::new(pos.x + glyph_pos.x + x_offset, {
                            let res = i64::from(pos.y + glyph_pos.y) - i64::from(glyph_size.y) - ymin;
                            if let Ok(val) = res.try_into() {
                                val
                            } else {
                                continue;
                            }
                        });

                        if (in_buffer_pos.x < buffer.size().x.get()) && (in_buffer_pos.y < buffer.size().y.get()) {
                            let glyph_intensity =
                                f32::from(glyph_bitmap[(glyph_pos.y * glyph_size.x + glyph_pos.x) as usize]) / 255.0;

                            if glyph_intensity > 0.0 {
                                buffer[in_buffer_pos] = self.color.scale(glyph_intensity).unwrap()
                                    + buffer[in_buffer_pos].scale(1.0 - glyph_intensity).unwrap();
                            }
                        }
                    }
                }

                x_offset += glyph_size.x + (self.fontsize / SPACE_BETWEEN_CHARS_MULT) as u32;
            }
        }
    }
}
