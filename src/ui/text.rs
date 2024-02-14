use std::vec;
use rusttype::point;
use crate::graphics::buffer::Buffer;
use crate::ui::font::Font;
use crate::graphics::object::GraphicsObject;
use crate::graphics::vertex::TexturedVertex;

pub struct Text<'a> {
    pub indices: Vec<u32>,
    pub vertices: Vec<TexturedVertex>,
    index_buffer: Buffer<'a>,
    vertex_buffer: Buffer<'a>,
    capacity: usize,
    length: usize
}

impl<'a> Text<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
                capacity: usize) -> Text<'a> {
        let offsets = vec![0, 1, 2, 0, 2, 3];
        let indices = (0..6 * capacity).map(|x| offsets[x % 6] + 4 * (x / 6) as u32).collect::<Vec<_>>();

        let vertices = vec![TexturedVertex {
            pos: [0.0, 0.0, 0., 0.],
            tex_coord: [0., 0.],
        }; 4 * capacity];

        let vertex_buffer = Buffer::new_vertex::<TexturedVertex>(vertices.len(), device, device_memory_properties);
        vertex_buffer.fill(&vertices);
        let index_buffer = Buffer::new_index(indices.len(), device, device_memory_properties);
        index_buffer.fill(&indices);

        Text {
            indices,
            vertices,
            vertex_buffer,
            index_buffer,
            capacity,
            length: 0,
        }
    }

    pub unsafe fn update(&mut self, content: &str, font: &Font) {
        if content.len() > self.capacity {
            println!("content is too long for capacity {}", self.capacity);
            return;
        }

        self.length = content.len();

        let glyphs: Vec<_> = font.rt_font.layout(content, font.scale, point(0., font.ascent)).collect();

        // create four textured vertices for each glyph
        for (i, glyph) in glyphs.iter().enumerate() {
            let rect = font.character_rect(&content.chars().nth(i).unwrap());
            // println!("character {:?} has rect {:?}", content.chars().nth(i), rect);

            if let Some(bb) = glyph.pixel_bounding_box() {
                // bottom left
                self.vertices[i * 4] = TexturedVertex {
                    pos: [bb.min.x as f32, bb.max.y as f32, 0., 1.],
                    tex_coord: [rect.min.x, rect.max.y],
                };
                // bottom right
                self.vertices[i * 4 + 1] = TexturedVertex {
                    pos: [bb.max.x as f32, bb.max.y as f32, 0., 1.],
                    tex_coord: [rect.max.x, rect.max.y],
                };
                // top right
                self.vertices[i * 4 + 2] = TexturedVertex {
                    pos: [bb.max.x as f32, bb.min.y as f32, 0., 1.],
                    tex_coord: [rect.max.x, rect.min.y],
                };
                // top left
                self.vertices[i * 4 + 3] = TexturedVertex {
                    pos: [bb.min.x as f32, bb.min.y as f32, 0., 1.],
                    tex_coord: [rect.min.x, rect.min.y],
                };
            } else {
                self.vertices[i * 4] = TexturedVertex::ZERO;
                self.vertices[i * 4 + 1] = TexturedVertex::ZERO;
                self.vertices[i * 4 + 2] = TexturedVertex::ZERO;
                self.vertices[i * 4 + 3] = TexturedVertex::ZERO;
            }
        }

        self.vertex_buffer.fill(&self.vertices);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl<'a> GraphicsObject<'a, TexturedVertex> for Text<'a> {
    fn index_buffer(&self) -> &Buffer<'a> {
        &self.index_buffer
    }

    fn vertex_buffer(&self) -> &Buffer<'a> {
        &self.vertex_buffer
    }

    fn vertices(&self) -> &Vec<TexturedVertex> {
        &self.vertices
    }

    fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}