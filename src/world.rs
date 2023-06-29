mod segment;
pub mod object;
pub mod ray;
pub mod size;
pub mod block;

use noise::{NoiseFn, Perlin};
use crate::graphics::object::GraphicsObject;
use object::*;
use size::*;
use segment::*;
use block::*;
use glam::Vec3;
use std::time;
use std::ops::Add;

// the indices of the triangles constituting the block face facing in negative x direction
const INDICES_NEG_X: [usize; 6] = [
    1, 0, 2,
    1, 2, 3,
];

const INDICES_POS_X: [usize; 6] = [
    4, 5, 6,
    5, 7, 6,
];

const INDICES_NEG_Y: [usize; 6] = [
    0, 1, 4,
    1, 5, 4,
];

const INDICES_POS_Y: [usize; 6] = [
    2, 6, 3,
    3, 6, 7,
];

const INDICES_NEG_Z: [usize; 6] = [
    0, 4, 2,
    2, 4, 6,
];

const INDICES_POS_Z: [usize; 6] = [
    1, 3, 5,
    3, 7, 5,
];

// const BL_INDICES: [usize; 36] = [
//     1, 0, 2,
//     1, 2, 3,
//     4, 5, 6,
//     5, 7, 6,
//     0, 1, 4,
//     1, 5, 4,
//     2, 6, 3,
//     3, 6, 7,
//     0, 4, 2,
//     2, 4, 6,
//     1, 3, 5,
//     3, 7, 5,
// ];

const BL_VERTICES: [[usize; 3]; 8] = [
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
    Z,
}

pub enum Face {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

impl Face {
    pub fn all() -> [Face; 6] {
        return [Face::XPos, Face::XNeg, Face::YPos, Face::YNeg, Face::ZPos, Face::ZNeg];
    }

    pub fn numeric(&self) -> ICoords {
        match self {
            Face::XPos => {
                return ICoords { x: 1, y: 0, z: 0 };
            }
            Face::XNeg => {
                return ICoords { x: -1, y: 0, z: 0 };
            }
            Face::YPos => {
                return ICoords { x: 0, y: 1, z: 0 };
            }
            Face::YNeg => {
                return ICoords { x: 0, y: -1, z: 0 };
            }
            Face::ZPos => {
                return ICoords { x: 0, y: 0, z: 1 };
            }
            Face::ZNeg => {
                return ICoords { x: 0, y: 0, z: -1 };
            }
        }
    }

    pub fn indices(&self) -> [usize; 6] {
        match self {
            Face::XPos => {
                INDICES_POS_X
            }
            Face::XNeg => {
                INDICES_NEG_X
            }
            Face::YPos => {
                INDICES_POS_Y
            }
            Face::YNeg => {
                INDICES_NEG_Y
            }
            Face::ZPos => {
                INDICES_POS_Z
            }
            Face::ZNeg => {
                INDICES_NEG_Z
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ICoords {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Add for ICoords {
    type Output = Self;
    fn add(self, rhs: ICoords) -> ICoords {
        ICoords { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl ICoords {
    fn decompose(&self, along: Dimension) -> (i64, i64, i64, i64) {
        match along {
            Dimension::X => {
                let l3c = (self.x / L3_SIZE_BL.x as i64) % L4_SIZE.x as i64;
                let l2c = (self.x / L2_SIZE_BL.x as i64) % L3_SIZE.x as i64;
                let l1c = (self.x / L1_SIZE_BL.x as i64) % L2_SIZE.x as i64;
                let bc = self.x % L1_SIZE.x as i64;
                return (l3c, l2c, l1c, bc);
            }
            Dimension::Y => {
                let l3c = (self.y / L3_SIZE_BL.y as i64) % L4_SIZE.y as i64;
                let l2c = (self.y / L2_SIZE_BL.y as i64) % L3_SIZE.y as i64;
                let l1c = (self.y / L1_SIZE_BL.y as i64) % L2_SIZE.y as i64;
                let bc = self.y % L1_SIZE.y as i64;
                return (l3c, l2c, l1c, bc);
            }
            Dimension::Z => {
                let l3c = (self.z / L3_SIZE_BL.z as i64) % L4_SIZE.z as i64;
                let l2c = (self.z / L2_SIZE_BL.z as i64) % L3_SIZE.z as i64;
                let l1c = (self.z / L1_SIZE_BL.z as i64) % L2_SIZE.z as i64;
                let bc = self.z % L1_SIZE.z as i64;
                return (l3c, l2c, l1c, bc);
            }
        }
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
            noise: Perlin::new(12),
            structure: L4Segment::default()
        };

        w.populate();

        return w;
    }

    fn populate(&mut self) {
        let now = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).expect("time went backwards");
        let t = (now.as_millis() % 10000) as f64;
        for x in 0 .. L2_SIZE_BL.x as i64 {
            for z in 0 .. L2_SIZE_BL.z as i64 {
                let y = (40. * self.noise.get([t, (x as f64) / 150., (z as f64) / 150.])).floor().max(0.);
                self.set_block(ICoords {x, y: y as i64, z}, BlockType::Grass);
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

    pub fn get_block(self, coords: ICoords) -> BlockType {
        let (l4x, l3x, l2x, l1x) = coords.decompose(Dimension::X);
        let (l4y, l3y, l2y, l1y) = coords.decompose(Dimension::Y);
        let (l4z, l3z, l2z, l1z) = coords.decompose(Dimension::Z);

        let l4coords = L4_SIZE.coordinates_1_d(l4x as usize, l4y as usize, l4z as usize);
        let l3coords = L3_SIZE.coordinates_1_d(l3x as usize, l3y as usize, l3z as usize);
        let l2coords = L2_SIZE.coordinates_1_d(l2x as usize, l2y as usize, l2z as usize);
        let l1coords = L1_SIZE.coordinates_1_d(l1x as usize, l1y as usize, l1z as usize);

        if let Some(l3) = &self.structure.sub_segments[l4coords] {
            if let Some(l2) = &l3.sub_segments[l3coords] {
                if let Some(l1) = &l2.sub_segments[l2coords] {
                    return l1.blocks[l1coords];
                }
            }
        }
        return BlockType::NoBlock;
    }

    pub fn set_block(&mut self, coords: ICoords, block: BlockType) {
        if coords.x >= L4_SIZE_BL.x as i64 || coords.y >= L4_SIZE_BL.y as i64 || coords.z >= L4_SIZE_BL.z as i64 {
            println!("[set_block]: coordinates ({}, {}, {}) are out of bounds", coords.x, coords.y, coords.z);
            return;
        }

        let (l4x, l3x, l2x, l1x) = coords.decompose(Dimension::X);
        let (l4y, l3y, l2y, l1y) = coords.decompose(Dimension::Y);
        let (l4z, l3z, l2z, l1z) = coords.decompose(Dimension::Z);

        let l4coords = L4_SIZE.coordinates_1_d(l4x as usize, l4y as usize, l4z as usize);
        let l3coords = L3_SIZE.coordinates_1_d(l3x as usize, l3y as usize, l3z as usize);
        let l2coords = L2_SIZE.coordinates_1_d(l2x as usize, l2y as usize, l2z as usize);
        let l1coords = L1_SIZE.coordinates_1_d(l1x as usize, l1y as usize, l1z as usize);

        let l3 = self.structure.sub_segments[l4coords].get_or_insert_with(L3Segment::default);
        let l2 = l3.sub_segments[l3coords].get_or_insert_with(L2Segment::default);
        let l1 = l2.sub_segments[l2coords].get_or_insert_with(L1Segment::default);
        l1.blocks[l1coords] = block;
    }
}