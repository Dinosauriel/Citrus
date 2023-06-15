mod segment;
pub mod object;
pub mod ray;
pub mod size;
pub mod block;

use noise::{NoiseFn, Perlin};
use crate::graphics::vertex::Vertex;
use crate::graphics::object::TriangleGraphicsObject;
use object::*;
use size::*;
use segment::*;
use block::*;
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

pub struct Coords {
    pub x: i64,
    pub y: i64,
    pub z: i64
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
            noise: Perlin::new(12),
            structure: L4Segment::default()
        };

        w.populate();

        return w;
    }

    fn populate(&mut self) {
        for x in 0 .. L2_SIZE_BL.x {
            for z in 0 .. L2_SIZE_BL.z {
                let y = (20. * self.noise.get([(x as f64) / 150., (z as f64) / 150.])).floor() as usize;
                self.set_block(x, y, z, BlockType::Grass);
            }
        }

        for (l3x, l3y, l3z) in L4_SIZE {
            if let Some(l3) = &self.structure.sub_segments[L4_SIZE.coordinates_1_d(l3x, l3y, l3z)] {

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