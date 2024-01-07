use std::vec;
use crate::ui::font::Font;
use crate::graphics::object::GraphicsObject;
use crate::graphics::vertex::TexturedVertex;


pub struct Text {
    pub content: String,
    pub indices: Vec<u32>,
    pub vertices: Vec<TexturedVertex>,
}

impl Text {
    pub fn new(content: &str, font: &Font) -> Text {
        let n = content.len();
        let offsets = vec![0, 1, 2, 0, 2, 3];
        let indices = (0..6 * n).map(|x| offsets[x % 6] + 4 * (x / 6) as u32).collect::<Vec<_>>();

        let glyphs: Vec<_> = font.rt_font.layout(content, font.scale, font.offset).collect();

        let mut vertices = vec![TexturedVertex {
            pos: [0.0, 0.0, 0., 0.],
            tex_coord: [0., 0.],
        }; 4 * n];

        // create four textured vertices for each glyph
        for (i, glyph) in glyphs.iter().enumerate() {
            let tex_position = font.character_position(&content.chars().nth(i).unwrap());
            println!("character {:?} has texture_position {:?}", content.chars().nth(i), tex_position);

            // TODO: proper font scaling!
            let pos = glyph.position();
            if let Some(bb) = glyph.pixel_bounding_box() {
                vertices[i * 4] = TexturedVertex {
                    pos: [pos.x, pos.y, 0., 1.],
                    // pos: [0., 1., 0., 2.],
                    tex_coord: [tex_position.0, tex_position.1],
                };
                vertices[i * 4 + 1] = TexturedVertex {
                    pos: [pos.x + bb.width() as f32, pos.y, 0., 1.],
                    // pos: [1., 1., 0., 2.],
                    tex_coord: [tex_position.0 + tex_position.2, tex_position.1],
                };
                vertices[i * 4 + 2] = TexturedVertex {
                    pos: [pos.x + bb.width() as f32, pos.y - bb.height() as f32, 0., 1.],
                    // pos: [1., 0., 0., 2.],
                    tex_coord: [tex_position.0 + tex_position.2, tex_position.1 - tex_position.3],
                };
                vertices[i * 4 + 3] = TexturedVertex {
                    pos: [pos.x, pos.y - bb.height() as f32, 0., 1.],
                    // pos: [0., 0., 0., 2.],
                    tex_coord: [tex_position.0, tex_position.1 - tex_position.3],
                };
            }
        }

        Text {
            content: String::from(content),
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