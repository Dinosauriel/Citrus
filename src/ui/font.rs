use crate::graphics::texture::Texture;
use std::{char, collections::HashMap, fs};
use rusttype::{point, PositionedGlyph, Rect, Scale};

pub const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHJKLMNOPQRSTUVWXYZ0123456789.,-+:;!?";

pub struct Font {
    pub font_size: usize,
    pub scale: Scale,
    pub ascent: f32, // height of the highest character in the font
    pub texture: Texture,
    pub rt_font: rusttype::Font<'static>,
    pub positions: HashMap<char, Rect<i32>>,
}

impl Font {
    pub fn load(g_state: &crate::graphics::state::GraphicState, path: &str, font_size: usize) -> Font {
        let data = fs::read(path).unwrap();
        let font = rusttype::Font::try_from_vec(data).unwrap();
        
        let scale = Scale {
            x: font_size as f32,
            y: font_size as f32,
        };

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        println!("font {:?} has v_metrics {:?}", path, v_metrics);

        // FOR REFERENCE:
        //                      ______ ascent line
        //       /\         
        //      /  \   __ _ 
        //     / /\ \ / _` |
        //    / ____ \ (_| |
        //   /_/    \_\__, |    _______ baseline
        //             __/ |
        //            |___/     _______ descent line

        let glyphs: Vec<PositionedGlyph<'_>> = font.layout(ALPHABET, scale, point(0.0, v_metrics.ascent)).collect();
        // todo: support overlapping fonts?
        // let g: Vec<_> = font.glyphs_for(ALPHABET.chars()).collect();

        let positions: HashMap<char, Rect<i32>> = ALPHABET.chars().zip(&glyphs).map(|(c, g)| (c, g.pixel_bounding_box().unwrap())).collect();

        // Find the most visually pleasing width to display
        // width of the entire "text"
        // take the last glyph in the layout and find its right border
        let width = glyphs.iter().rev().map(
                |g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next().unwrap().ceil() as usize;

        // rasterize to row major buffer
        let mut pixels = vec![0; width * font_size * 4];
        for g in glyphs {
            let bb = g.pixel_bounding_box().unwrap();
            g.draw(|x, y, v| {
                let alpha = (v * 255.) as u8;
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if (0..width as i32).contains(&x) && (0..font_size as i32).contains(&y) {
                    let x = x as usize;
                    let y = y as usize;
                    let px = 4 * (y * width + x);
                    pixels[px + 0] = 0xff;
                    pixels[px + 1] = 0xff;
                    pixels[px + 2] = 0xff;
                    pixels[px + 3] = alpha;
                }
            })
        }

        unsafe {
            let texture = Texture::create_from_bytes(g_state, &pixels, width as u32, font_size as u32);

            Font {
                font_size,
                scale,
                ascent: v_metrics.ascent,
                texture,
                rt_font: font,
                positions
            }
        }
    }

    pub fn character_rect(&self, c: &char) -> Rect<f32> {
        if let Some(rect) = self.positions.get(c) {
            return Rect {
                min: point(rect.min.x as f32 / self.texture.image.width as f32, rect.min.y as f32 / self.texture.image.height as f32),
                max: point(rect.max.x as f32 / self.texture.image.width as f32, rect.max.y as f32 / self.texture.image.height as f32),
            }
        }
        // println!("character {} not in alphabet!", c);
        Rect {min: point(0., 0.), max: point(0., 0.)}
    }
}
