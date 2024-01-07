use crate::graphics::texture::Texture;
use std::{fs, char};
use rusttype::{point, Point, Scale};

pub const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHJKLMNOPQRSTUVWXYZ0123456789";

pub struct Font {
    pub scale: Scale,
    pub offset: Point<f32>,
    pub texture: Texture,
    pub rt_font: rusttype::Font<'static>,
    pub positions: Vec<(f32, f32, f32, f32)>, 
}

impl Font {
    pub fn load(g_state: &crate::graphics::state::GraphicState, path: &str, font_size: usize) -> Font {
        let data = fs::read(path).unwrap();
        let font = rusttype::Font::try_from_vec(data).unwrap();
        
        // 2x scale in x direction to counter the aspect ratio of monospace characters.
        let scale = Scale {
            x: font_size as f32 * 2.0,
            y: font_size as f32,
        };

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        // Glyphs to draw for "RustType". Feel free to try other strings.
        let glyphs: Vec<_> = font.layout(ALPHABET, scale, offset).collect();

        // Find the most visually pleasing width to display
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        let positions: Vec<_> = glyphs.iter().map(|g| 
                                                (g.position().x, g.position().y, g.pixel_bounding_box().unwrap().width() as f32, g.pixel_bounding_box().unwrap().height() as f32)).collect();

        println!("width: {}, height: {}", width, font_size);

        // rasterize to row major buffer
        let mut pixels = vec![0; width * font_size * 4];
        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let alpha = (v * 255.) as u8;
                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;
                    // There's still a possibility that the glyph clips the boundaries of the bitmap
                    if (0..width as i32).contains(&x) && (0..font_size as i32).contains(&y) {
                        let x = x as usize;
                        let y = y as usize;
                        let px = 4 * (y * width + x);
                        pixels[px + 0] = 255;
                        pixels[px + 1] = 255;
                        pixels[px + 2] = 255;
                        pixels[px + 3] = alpha;
                    }
                })
            }
        }

        unsafe {
            let texture = Texture::create_from_bytes(g_state, &pixels, width as u32, font_size as u32);

            Font {
                scale,
                offset,
                texture,
                rt_font: font,
                positions
            }
        }
    }

    pub fn character_position(&self, character: &char) -> (f32, f32, f32, f32) {
        // find position of char in alphabet
        if let Some(j) = ALPHABET.chars().position(|x| &x == character) {
            println!("character {character} has index {j}");

            return (
                self.positions[j].0 / self.texture.image.width as f32,
                self.positions[j].1 / self.texture.image.height as f32,
                self.positions[j].2 / self.texture.image.width as f32,
                self.positions[j].3 / self.texture.image.height as f32,
            );
        }

        return (0., 0., 0., 0.);
    }
}
