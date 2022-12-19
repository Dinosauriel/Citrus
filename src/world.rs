mod segment;
pub mod size;

use noise::{NoiseFn, Perlin};
use crate::graphics::object::Vertex;
use crate::graphics::object::TriangleGraphicsObject;
use size::*;
use segment::*;

use glam::Vec3;

const BLOCK_TRIANGLE_INDICES: [usize; 36] = [
    1, 0, 2,
    1, 2, 3,
    4, 5, 6,
    5, 7, 6,
    0, 1, 4,
    1, 5, 4,
    2, 6, 3,
    3, 6, 7,
    0, 4, 2,
    2, 4, 6,
    1, 3, 5,
    3, 7, 5,
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

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum BlockType {
    NoBlock,
    Grass
}

impl Default for BlockType {
    fn default() -> Self {
        return Self::NoBlock;
    }
}

pub struct BlockObject {
    position: Vec3,
    size: Size3D,
    blocks: Vec<BlockType>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl BlockObject {
    pub fn new(size: Size3D, position: Vec3) -> Self {
        let mut obj = BlockObject {
            position: position,
            size: size,
            blocks: Vec::with_capacity(size.volume()),
            vertices: Vec::with_capacity(size.num_vertices()),
            indices: Vec::with_capacity(36 * size.volume()),
        };

        obj.blocks.resize(obj.blocks.capacity(), BlockType::Grass);
        obj.indices.resize(obj.indices.capacity(), 0);
        obj.vertices.resize_with(obj.vertices.capacity(), Default::default);

        obj.update_vertices();
        obj.update_indices();

        return obj;
    }

    fn update_indices(&mut self) {
        self.indices.fill(0);
        for x in 0 .. self.size.x {
            for y in 0 .. self.size.y {
                for z in 0 .. self.size.z {
                    let coordinates = self.size.coordinates_1_d(x, y, z);
                    if self.blocks[coordinates] == BlockType::Grass {
                        for i in 0 .. 36 {
                            let [d_x, d_y, d_z] = BLOCK_VERTICES[BLOCK_TRIANGLE_INDICES[i]];
                            self.indices[coordinates * 36 + i] = self.size.vertex_coordinates_1_d(x + d_x, y + d_y, z + d_z) as u32;
                        }
                    }
                }
            }
        }
    }

    fn update_vertices(&mut self) {
        for x in 0 .. self.size.x + 1 {
            for y in 0 .. self.size.y + 1 {
                for z in 0 .. self.size.z + 1 {
                    let vertex = Vertex {
                        pos: [x as f32 + self.position.x, y as f32 + self.position.x, z as f32 + self.position.x, 1.0],
                        color: [1.0, 1.0, 0.0, 1.0]
                    };
                    self.vertices[self.size.vertex_coordinates_1_d(x, y, z)] = vertex;
                }
            }
        }
    }

    pub fn num_blocks(&self) -> u32 {
        let mut n = 0;
        for block in self.blocks.iter() {
            if block != &BlockType::NoBlock {
                n += 1;
            }
        }
        return n;
    }
}

impl TriangleGraphicsObject for BlockObject {
    fn vertices(&self) -> &Vec<Vertex> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}

pub struct World {
    pub objects: Vec<BlockObject>,
    noise: Perlin,
    pub structure: L4Segment,
}

impl World {
    pub fn new() -> Self {
        let mut w = World {
            objects: Vec::new(),
            noise: Perlin::new(),
            structure: L4Segment::default()
        };

        w.populate();

        return w;
    }

    fn populate(&mut self) {
        let mut l1_seg = L1Segment::default();
        let mut l2_seg = L2Segment::default();
        let mut l3_seg = L3Segment::default();
        
        for x in 0 .. L1_SEGMENT_SIZE.x {
            for z in 0 .. L1_SEGMENT_SIZE.z {
                let y = (5. * self.noise.get([(x as f64) / 10., (z as f64) / 10.])).floor() as usize;
                assert!(y < L1_SEGMENT_SIZE.y);
                let coords = L1_SEGMENT_SIZE.coordinates_1_d(x, y, z);
                
                l1_seg.blocks[coords] = BlockType::Grass;
            }
        }

        l2_seg.sub_segments[0] = Some(l1_seg);
        l3_seg.sub_segments[0] = Some(l2_seg);
        self.structure.sub_segments[0] = Some(l3_seg);
    }
}