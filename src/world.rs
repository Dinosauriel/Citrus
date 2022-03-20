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

const BLOCK_VERTICES: [[usize; 3]; 8] = [
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 1],

    [1, 0, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 1, 1]
];

#[derive(Clone)]
pub enum BlockType {
    NoBlock,
    Grass
}

pub struct World {
    pub size: usize,
    pub height: usize,
    blocks: Vec<BlockType>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    noise: Perlin,
}

impl World {
    fn populate(&mut self) {
        for x in 0 .. self.size {
            for z in 0 .. self.size {
                let y = (5. * self.noise.get([(x as f64) / 10., (z as f64) / 10.])).floor() as usize;
                assert!(y < self.height);
                let coords = self.block_coordinates(&x, &y, &z);
                self.blocks[coords] = BlockType::Grass;

                for i in 0 .. BLOCK_TRIANGLE_INDICES.len() {
                    let [v_x, v_y, v_z] = BLOCK_VERTICES[BLOCK_TRIANGLE_INDICES[i]];
                    self.indices[(x * self.size + z) * 36 + i] = ((y + v_y) * (self.size + 1) * (self.size + 1) + (x + v_x) * (self.size + 1) + (z + v_z)) as u32;
                }
            }
        }

        for y in 0 .. self.height + 1 {
            for x in 0 .. self.size + 1 {
                for z in 0 .. self.size + 1 {
                    self.vertices[y * (self.size + 1) * (self.size + 1) + x * (self.size + 1) + z] = Vertex {
                        pos: [x as f32, y as f32, z as f32, 1.0],
                        color: [0.0, 1.0, 0.0, 1.0]
                    }
                }
            }
        }


        // for i in 0 .. self.blocks.len() {
        //     let block = self.blocks[i];
        //     for x in 0 .. 2 {
        //         for y in 0 .. 2 {
        //             for z in 0 .. 2 {
        //                 let [block_x, block_y, block_z] = block;
        //                 self.vertices[8 * i + 4 * x + 2 * y + z] = Vertex {
        //                     pos: [(block_x + x as u32) as f32, (block_y + y as u32) as f32, (block_z + z as u32) as f32, 1.0],
        //                     color: [0.0, 1.0, 0.0, 1.0]
        //                 };
        //             }
        //         }
        //     }
            
        //     for j in 0 .. BLOCK_TRIANGLE_INDICES.len() {
        //         self.indices[i * 36 + j] = (8 * i + BLOCK_TRIANGLE_INDICES[j]) as u32;
        //     }
        // }
    }

    pub fn new(size: usize, height: usize) -> Self {

        let mut w = World {
            size: size,
            height: height,
            blocks: Vec::with_capacity(size * size * height),
            vertices: Vec::with_capacity((size + 1) * (size + 1) * (height + 1)),
            indices: Vec::with_capacity(6 * 6 * size * size),
            noise: Perlin::new(),
        };

        w.blocks.resize(w.blocks.capacity(), BlockType::NoBlock);
        w.indices.resize(w.indices.capacity(), 0);
        w.vertices.resize_with(w.vertices.capacity(), Default::default);

        w.populate();

        return w;
    }

    pub fn block_coordinates(&self, x: &usize, y: &usize, z: &usize) -> usize {
        return y * self.size * self.size + x * self.size + z;
    }

    pub fn block_at(&self, x: &usize, y: &usize, z: &usize) -> &BlockType {
        if *x >= self.size || *y >= self.height || *z >= self.size {
            return &BlockType::NoBlock;
        }
        return &self.blocks[self.block_coordinates(x, y, z)];
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