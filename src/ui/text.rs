use std::vec;
use std::fs;
use rusttype::{point, Scale};
use crate::graphics::object::GraphicsObject;
use crate::graphics::texture::Texture;
use crate::graphics::vertex::TexturedVertex;

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHJKLMNOPQRSTUVWXYZ0123456789";

pub struct Text {
    content: String,
    font: &'static Font,
}

impl Text {
    pub fn new(content: &str, font: &'static Font) -> Text {
        Text {
            content: String::from(content),
            font,
        }
    }
}

// impl GraphicsObject<TexturedVertex> for Text {
//     fn indices(&self) -> &Vec<u32> {
        
//     }

//     fn vertices(&self) -> &Vec<TexturedVertex> {
        
//     }
// }

pub struct Font {
    name: String,
    texture: Texture,
    rt_font: rusttype::Font<'static>,
    positions: Vec<(f32, f32, f32, f32)>, 
}

pub unsafe fn load_font(path: &str) -> (Vec<u8>, usize, usize) {
    let data = fs::read(path).unwrap();
    let font = rusttype::Font::try_from_vec(data).unwrap();

    // Desired font pixel height
    let height: usize = 72;

    // 2x scale in x direction to counter the aspect ratio of monospace characters.
    let scale = Scale {
        x: height as f32 * 2.0,
        y: height as f32,
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

    let positions: Vec<_> = glyphs.iter().map(|g| { (g.position().x, g.position().y, g.scale().x, g.scale().y) }).collect();

    println!("width: {}, height: {}", width, height);

    // rasterize to row major buffer
    let mut pixels = vec![0; width * height * 4];
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let alpha = (v * 255.) as u8;
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
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

    return (pixels, width, height);
}
