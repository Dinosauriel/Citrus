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

pub enum Dimension {
    X,
    Y,
    Z
}

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
                        pos: [x as f32 + self.position.x, y as f32 + self.position.y, z as f32 + self.position.z, 1.0],
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
        for x in 0 .. L2_SIZE_BL.x {
            for z in 0 .. L2_SIZE_BL.z {
                let y = (5. * self.noise.get([(x as f64) / 10., (z as f64) / 10.])).floor() as usize;
                self.set_block(x, y, z, BlockType::Grass);
            }
        }

        println!("blocks set");

        for (l3x, l3y, l3z) in L4_SIZE {
            print!("{:?} contains ", (l3x, l3y, l3z));
            if let Some(l3) = &self.structure.sub_segments[L4_SIZE.coordinates_1_d(l3x, l3y, l3z)] {
                println!("Some");
                for (l2x, l2y, l2z) in L3_SIZE {
                    if let Some(l2) = &l3.sub_segments[L3_SIZE.coordinates_1_d(l2x, l2y, l2z)] {
                        for (l1x, l1y, l1z) in L2_SIZE {
                            if let Some(l1) = &l2.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z)] {
                                let x_offset = l3x * L3_SIZE_BL.x + l2x * L2_SIZE_BL.x + l1x * L1_SIZE_BL.x;
                                let y_offset = l3y * L3_SIZE_BL.y + l2y * L2_SIZE_BL.y + l1y * L1_SIZE_BL.y;
                                let z_offset = l3z * L3_SIZE_BL.z + l2z * L2_SIZE_BL.z + l1z * L1_SIZE_BL.z;

                                let o = l1.object(Vec3::new(x_offset as f32, y_offset as f32, z_offset as f32));
                                self.objects.push(o);
                            }
                        }
                    }
                }
            } else {
                println!("None");
            }
        }
    }

    pub fn get_block(self, x: usize, y: usize, z: usize) -> BlockType {
        let (l4x, l3x, l2x, l1x) = Self::decompose(x, Dimension::X);
        let (l4y, l3y, l2y, l1y) = Self::decompose(y, Dimension::Y);
        let (l4z, l3z, l2z, l1z) = Self::decompose(z, Dimension::Z);

        let l4coords = L4_SIZE.coordinates_1_d(l4x, l4y, l4z);
        let l3coords = L3_SIZE.coordinates_1_d(l3x, l3y, l3z);
        let l2coords = L2_SIZE.coordinates_1_d(l2x, l2y, l2z);
        let l1coords = L1_SIZE.coordinates_1_d(l1x, l1y, l1z);

        if let Some(l3) = &self.structure.sub_segments[l4coords] {
            if let Some(l2) = &l3.sub_segments[l3coords] {
                if let Some(l1) = &l2.sub_segments[l2coords] {
                    return l1.blocks[l1coords];
                }
            }
        }
        return BlockType::NoBlock;
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x >> 12 > 0 || y >> 12 > 0 || z >> 12 > 0 {
            println!("[set_block]: coordinates ({x}, {y}, {z}) are out of bounds");
            return;
        }

        let (l4x, l3x, l2x, l1x) = Self::decompose(x, Dimension::X);
        let (l4y, l3y, l2y, l1y) = Self::decompose(y, Dimension::Y);
        let (l4z, l3z, l2z, l1z) = Self::decompose(z, Dimension::Z);

        let l4coords = L4_SIZE.coordinates_1_d(l4x, l4y, l4z);
        let l3coords = L3_SIZE.coordinates_1_d(l3x, l3y, l3z);
        let l2coords = L2_SIZE.coordinates_1_d(l2x, l2y, l2z);
        let l1coords = L1_SIZE.coordinates_1_d(l1x, l1y, l1z);

        let l3 = self.structure.sub_segments[l4coords].get_or_insert_with(L3Segment::default);
        let l2 = l3.sub_segments[l3coords].get_or_insert_with(L2Segment::default);
        let l1 = l2.sub_segments[l2coords].get_or_insert_with(L1Segment::default);
        l1.blocks[l1coords] = block;
    }

    fn decompose(c: usize, along: Dimension) -> (usize, usize, usize, usize) {
        let l3c;
        let l2c;
        let l1c;
        let bc;
        match along {
            Dimension::X => {
                l3c = (c / L3_SIZE_BL.x) % L4_SIZE.x;
                l2c = (c / L2_SIZE_BL.x) % L3_SIZE.x;
                l1c = (c / L1_SIZE_BL.x) % L2_SIZE.x;
                bc = c % L1_SIZE.x;
            }
            Dimension::Y => {
                l3c = (c / L3_SIZE_BL.y) % L4_SIZE.y;
                l2c = (c / L2_SIZE_BL.y) % L3_SIZE.y;
                l1c = (c / L1_SIZE_BL.y) % L2_SIZE.y;
                bc = c % L1_SIZE.y;
            }
            Dimension::Z => {
                l3c = (c / L3_SIZE_BL.z) % L4_SIZE.z;
                l2c = (c / L2_SIZE_BL.z) % L3_SIZE.z;
                l1c = (c / L1_SIZE_BL.z) % L2_SIZE.z;
                bc = c % L1_SIZE.z;
            }
        }
        return (l3c, l2c, l1c, bc);
    }
}