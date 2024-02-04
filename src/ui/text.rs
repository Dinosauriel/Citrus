use std::vec;
use rusttype::point;
use crate::ui::font::Font;
use crate::graphics::object::GraphicsObject;
use crate::graphics::vertex::TexturedVertex;

pub struct Text {
    pub indices: Vec<u32>,
    pub vertices: Vec<TexturedVertex>,
}

impl Text {
    pub fn new(content: &str, font: &Font) -> Text {
        let offsets = vec![0, 1, 2, 0, 2, 3];
        let indices = (0..6 * content.len()).map(|x| offsets[x % 6] + 4 * (x / 6) as u32).collect::<Vec<_>>();

        let glyphs: Vec<_> = font.rt_font.layout(content, font.scale, point(0., font.ascent)).collect();

        let mut vertices = vec![TexturedVertex {
            pos: [0.0, 0.0, 0., 0.],
            tex_coord: [0., 0.],
        }; 4 * content.len()];

        // create four textured vertices for each glyph
        for (i, glyph) in glyphs.iter().enumerate() {
            let rect = font.character_rect(&content.chars().nth(i).unwrap());
            println!("character {:?} has rect {:?}", content.chars().nth(i), rect);

            if let Some(bb) = glyph.pixel_bounding_box() {
                // bottom left
                vertices[i * 4] = TexturedVertex {
                    pos: [bb.min.x as f32, bb.max.y as f32, 0., 1.],
                    tex_coord: [rect.min.x, rect.max.y],
                };
                // bottom right
                vertices[i * 4 + 1] = TexturedVertex {
                    pos: [bb.max.x as f32, bb.max.y as f32, 0., 1.],
                    tex_coord: [rect.max.x, rect.max.y],
                };
                // top right
                vertices[i * 4 + 2] = TexturedVertex {
                    pos: [bb.max.x as f32, bb.min.y as f32, 0., 1.],
                    tex_coord: [rect.max.x, rect.min.y],
                };
                // top left
                vertices[i * 4 + 3] = TexturedVertex {
                    pos: [bb.min.x as f32, bb.min.y as f32, 0., 1.],
                    tex_coord: [rect.min.x, rect.min.y],
                };
            }
        }

        Text {
            indices,
            vertices
        }
    }
}

impl GraphicsObject<TexturedVertex> for Text {
    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }

    fn vertices(&self) -> &Vec<TexturedVertex> {
        return &self.vertices;
    }
}