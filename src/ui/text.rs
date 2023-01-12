use std::vec;
use std::fs;
use rusttype::{point, Font, Scale};
use crate::graphics::state::GraphicState;
use crate::graphics::texture;

pub unsafe fn load_font(g_state: &GraphicState, path: &str) -> texture::Texture {
    let data = fs::read(path).unwrap();
    let font = Font::try_from_vec(data).unwrap();

    // Desired font pixel height
    let height: f32 = 48.;
    let pixel_height = height.ceil() as usize;

    // 2x scale in x direction to counter the aspect ratio of monospace characters.
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    // Glyphs to draw for "RustType". Feel free to try other strings.
    let glyphs: Vec<_> = font.layout("__HI__there", scale, offset).collect();

    // Find the most visually pleasing width to display
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    // rasterize to row major buffer
    let mut pixel_data = vec![0; width * pixel_height * 4];
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let alpha = (v * 255.) as u8;
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    let px = 4 * (y * width + x);
                    pixel_data[px + 0] = 255;
                    pixel_data[px + 1] = 255;
                    pixel_data[px + 2] = 255;
                    pixel_data[px + 3] = alpha;
                }
            })
        }
    }

    return texture::Texture::create_from_bytes(g_state, &pixel_data, width as u32, pixel_height as u32);
}


