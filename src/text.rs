use cgmath::Vector2;
use fontdue::{Font, FontSettings};
use lazy_static::lazy_static;

use crate::framebuffer::{Color, Draw, FrameBuffer};

pub struct Text(pub String);

static FONT_BYTES: &[u8] = include_bytes!("../resource/JetBrainsMonoNerdFont-Regular.ttf");
lazy_static! {
    static ref FONT: Font = Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap();
}

impl Text {
    pub fn new(text: impl ToString) -> Self {
        Self(text.to_string())
    }
}

impl Draw for Text {
    fn draw(&self, pos: Vector2<u32>, buffer: &mut FrameBuffer) {
        let mut x_offset = 0;
        for char in self.0.chars() {
            let (glyph_size, glyph_bitmap) = {
                let (metrics, bitmap) = FONT.rasterize(char, 40.0);

                (Vector2::new(metrics.width as u32, metrics.height as u32), bitmap)
            };

            for x in 0..glyph_size.x {
                for y in 0..glyph_size.y {
                    let glyph_pos = Vector2::new(x, y);
                    let in_buffer_pos = Vector2::new(pos.x + glyph_pos.x + x_offset, pos.y + glyph_pos.y);

                    if (in_buffer_pos.x < buffer.size().x) && (in_buffer_pos.y < buffer.size().y) {
                        let glyph_intensity = glyph_bitmap[(glyph_pos.y * glyph_size.x + glyph_pos.x) as usize];
                        if glyph_intensity > 0 {
                            *buffer[in_buffer_pos] = *Color::RED;
                        }
                    }
                }
            }

            x_offset += glyph_size.x;
        }
    }
}
