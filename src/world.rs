use noise::{NoiseFn, Perlin};
use crate::graphics::object::Vertex;
use crate::graphics::object::TriangleGraphicsObject;

const BLOCK_TRIANGLE_INDICES: [usize; 36] = [
    0, 1, 2,
    1, 2, 3,
    4, 5, 6,
    5, 6, 7,
    0, 1, 4,
    1, 4, 5,
    2, 3, 6,
    3, 6, 7,
    0, 2, 4,
    2, 4, 6,
    1, 3, 5,
    3, 5, 7,
];

pub struct World {
    pub size: usize,
    blocks: Vec<[u32; 3]>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    noise: Perlin,
}

impl World {
    fn populate(&mut self) {
        for x in 0 .. self.size {
            for z in 0 .. self.size {
                let y = (5. * self.noise.get([(x as f64) / 10., (z as f64) / 10.])).floor() as u32;
                self.blocks[x * self.size + z] = [x as u32, y, z as u32];
            }
        }


        for i in 0 .. self.blocks.len() {
            let block = self.blocks[i];
            for x in 0 .. 2 {
                for y in 0 .. 2 {
                    for z in 0 .. 2 {
                        let [block_x, block_y, block_z] = block;
                        self.vertices[8 * i + 4 * x + 2 * y + z] = Vertex {
                            pos: [(block_x + x as u32) as f32, (block_y + y as u32) as f32, (block_z + z as u32) as f32, 1.0],
                            color: [0.0, 1.0, 0.0, 1.0]
                        };
                    }
                }
            }
            
            for j in 0 .. BLOCK_TRIANGLE_INDICES.len() {
                self.indices[i * 36 + j] = (8 * i + BLOCK_TRIANGLE_INDICES[j]) as u32;
            }
        }
    }

    pub fn new(size: usize) -> Self {
        let number_of_blocks = size * size;

        let mut w = World {
            size: size,
            blocks: Vec::with_capacity(number_of_blocks),
            vertices: Vec::with_capacity(8 * number_of_blocks),
            indices: Vec::with_capacity(6 * 6 * number_of_blocks),
            noise: Perlin::new(),
        };

        w.blocks.resize_with(w.blocks.capacity(), Default::default);
        w.indices.resize_with(w.indices.capacity(), Default::default);
        w.vertices.resize_with(w.vertices.capacity(), Default::default);

        w.populate();

        return w;
    }
}

impl TriangleGraphicsObject for World {
    fn vertices(&self) -> &Vec<Vertex> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}