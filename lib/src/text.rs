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
    pub fn new(text: impl ToString, fontsize: f32, color: Option<Color>) -> Self {
        let text = text.to_string();

        assert!(fontsize.is_finite() && fontsize > 0.0);
        assert!(!text.is_empty());

        Self {
            text,
            color: color.unwrap_or_default(),
            fontsize,
        }
    }
}

impl Draw for Label {
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
                        metrics.ymin as i64,
                        bitmap,
                    )
                };

                for x in 0..glyph_size.x {
                    for y in 0..glyph_size.y {
                        let glyph_pos = Vector2::new(x, y);
                        let in_buffer_pos = Vector2::new(pos.x + glyph_pos.x + x_offset, {
                            let res = ((pos.y + glyph_pos.y) as i64) - (glyph_size.y as i64) - ymin;
                            if let Ok(val) = res.try_into() {
                                val
                            } else {
                                continue;
                            }
                        });

                        if (in_buffer_pos.x < buffer.size().x) && (in_buffer_pos.y < buffer.size().y) {
                            let glyph_intensity =
                                (glyph_bitmap[(glyph_pos.y * glyph_size.x + glyph_pos.x) as usize] as f32) / 255.0;

                            if glyph_intensity > 0.0 {
                                buffer[in_buffer_pos] = self.color.scale(glyph_intensity)
                                    + buffer[in_buffer_pos].scale(1.0 - glyph_intensity);
                            }
                        }
                    }
                }

                x_offset += glyph_size.x + (self.fontsize / SPACE_BETWEEN_CHARS_MULT) as u32;
            }
        }
    }
}
