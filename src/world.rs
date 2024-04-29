mod segment;
pub mod object;
pub mod ray;
pub mod size;
pub mod block;

use std::time;
use std::ops::Add;
use std::collections::HashMap;
use noise::{NoiseFn, Perlin};
use glam::Vec3;
use crate::graphics::graphics_object::GraphicsObject;
use object::*;
use size::*;
use segment::*;
use block::*;

// the indices of the triangles constituting the block face facing in negative x direction
const INDICES_NEG_X: [u64; 6] = [
    1, 0, 2,
    1, 2, 3,
];

const INDICES_POS_X: [u64; 6] = [
    4, 5, 6,
    5, 7, 6,
];

const INDICES_NEG_Y: [u64; 6] = [
    0, 1, 4,
    1, 5, 4,
];

const INDICES_POS_Y: [u64; 6] = [
    2, 6, 3,
    3, 6, 7,
];

const INDICES_NEG_Z: [u64; 6] = [
    0, 4, 2,
    2, 4, 6,
];

const INDICES_POS_Z: [u64; 6] = [
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

const BL_VERTICES: [[u64; 3]; 8] = [
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 1],

    [1, 0, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 1, 1]
];

pub enum Axis {
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

    pub fn indices(&self) -> [u64; 6] {
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

#[derive(Clone, Copy, Debug)]
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
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// for a given world coordinate, find the coordinates of the respective L3, L2 and L1 segments that contain this coordinate
    /// - along: the axis that should be decomposed
    fn decompose(&self, along: Axis) -> (i64, i64, i64) {
        match along {
            Axis::X => {
                let l2c = (self.x / L2_SIZE_BL.x as i64) % L3_SIZE.x as i64;
                let l1c = (self.x / L1_SIZE_BL.x as i64) % L2_SIZE.x as i64;
                let bc = self.x % L1_SIZE.x as i64;
                return (l2c, l1c, bc);
            }
            Axis::Y => {
                let l2c = (self.y / L2_SIZE_BL.y as i64) % L3_SIZE.y as i64;
                let l1c = (self.y / L1_SIZE_BL.y as i64) % L2_SIZE.y as i64;
                let bc = self.y % L1_SIZE.y as i64;
                return (l2c, l1c, bc);
            }
            Axis::Z => {
                let l2c = (self.z / L2_SIZE_BL.z as i64) % L3_SIZE.z as i64;
                let l1c = (self.z / L1_SIZE_BL.z as i64) % L2_SIZE.z as i64;
                let bc = self.z % L1_SIZE.z as i64;
                return (l2c, l1c, bc);
            }
        }
    }

    /// coordinates of the L3 segment that contains self
    pub fn l3_coords(&self) -> (i64, i64, i64) {
        let x = self.x.div_euclid(L3_SIZE_BL.x as i64);
        let y = self.y.div_euclid(L3_SIZE_BL.y as i64);
        let z = self.z.div_euclid(L3_SIZE_BL.z as i64);
        (x, y, z)
    }

    /// coordinates of the L2 segment that contains self
    pub fn l2_coords(&self) -> (u64, u64, u64) {
        let x = self.x.div_euclid(L2_SIZE_BL.x as i64).rem_euclid(L3_SIZE.x as i64) as u64;
        let y = self.y.div_euclid(L2_SIZE_BL.y as i64).rem_euclid(L3_SIZE.y as i64) as u64;
        let z = self.z.div_euclid(L2_SIZE_BL.z as i64).rem_euclid(L3_SIZE.z as i64) as u64;
        (x, y, z)
    }

    /// coordinates of the L1 segment that contains self
    pub fn l1_coords(&self) -> (u64, u64, u64) {
        let x = self.x.div_euclid(L1_SIZE_BL.x as i64).rem_euclid(L2_SIZE.x as i64) as u64;
        let y = self.y.div_euclid(L1_SIZE_BL.y as i64).rem_euclid(L2_SIZE.y as i64) as u64;
        let z = self.z.div_euclid(L1_SIZE_BL.z as i64).rem_euclid(L2_SIZE.z as i64) as u64;
        (x, y, z)
    }

    /// coordinates of self within the L1 segment
    pub fn local_coords(&self) -> (u64, u64, u64) {
        let x = self.x.rem_euclid(L1_SIZE_BL.x as i64) as u64;
        let y = self.y.rem_euclid(L1_SIZE_BL.y as i64) as u64;
        let z = self.z.rem_euclid(L1_SIZE_BL.z as i64) as u64;
        (x, y, z)
    }
}

pub struct World<'a> {
    pub objects: Vec<BlockObject<'a>>,
    pub terrain: HashMap<(i64, i64, i64), L3Segment>,
    seed: u32,
}

impl<'a> World<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) -> Self {
        let mut w = World {
            objects: Vec::new(),
            terrain: HashMap::new(),
            seed: 12,
        };

        w.terrain.insert((0, 0, 0), L3Segment::default());
        // w.populate(device, device_memory_properties);

        for (l1x, l1y, l1z) in L2_SIZE {
            w.generate_l1_segment(ICoords::new((l1x * L1_SIZE_BL.x) as i64, (l1y * L1_SIZE_BL.y) as i64, (l1z * L1_SIZE_BL.z) as i64));
        }

        w.generate_graphics_objects(device, device_memory_properties);
        w
    }

    fn l3_segment(&self, coords: ICoords) -> Option<&L3Segment> {
        self.terrain.get(&coords.l3_coords())
    }

    fn create_or_get_l3(&mut self, coords: ICoords) -> &mut L3Segment {
        if !self.terrain.contains_key(&coords.l3_coords()) {
            self.terrain.insert(coords.l3_coords(), L3Segment::default());
        }
        self.terrain.get_mut(&coords.l3_coords()).unwrap()
    }

    fn l2_segment(&self, coords: ICoords) -> Option<&L2Segment> {
        if let Some(l3_seg) = self.l3_segment(coords) {
            let (l2x, l2y, l2z) = coords.l3_coords();
            return l3_seg.sub_segments[L2_SIZE.coordinates_1_d(l2x as u64, l2y as u64, l2z as u64) as usize].as_ref()
        }

        None
    }

    fn create_or_get_l2(&mut self, coords: ICoords) -> &mut L2Segment {
        let l3_seg = self.create_or_get_l3(coords);
        let (l2x, l2y, l2z) = coords.l2_coords();
        if l3_seg.sub_segments[L2_SIZE.coordinates_1_d(l2x, l2y, l2z) as usize].is_none() {
            l3_seg.sub_segments[L2_SIZE.coordinates_1_d(l2x, l2y, l2z) as usize] = Some(L2Segment::default());
        }

        l3_seg.sub_segments[L2_SIZE.coordinates_1_d(l2x, l2y, l2z) as usize].as_mut().unwrap()
    }

    fn create_or_get_l1(&mut self, coords: ICoords) -> &mut L1Segment {
        let l2_seg = self.create_or_get_l2(coords);
        let (l1x, l1y, l1z) = coords.l1_coords();
        if l2_seg.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z) as usize].is_none() {
            l2_seg.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z) as usize] = Some(L1Segment::default());
        }

        l2_seg.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z) as usize].as_mut().unwrap()
    }

    /// * `coords` - coordinates of the 0 0 0 block in the desired l1_segment
    fn generate_l1_segment(&mut self, coords: ICoords) {
        let noise = Perlin::new(self.seed);
        let l1_seg = self.create_or_get_l1(coords);

        println!("[generate_l1_segment]: {:?}", coords);
        for (d_x, d_y, d_z) in L1_SIZE_BL {
            let d_c = ICoords::new(coords.x + d_x as i64, coords.y + d_y as i64, coords.z + d_z as i64);
            let v = noise.get([(d_c.x as f64) / 50., (d_c.y as f64) / 50., (d_c.z as f64) / 50.]);
            if v > 0. {
                l1_seg.blocks[L1_SIZE_BL.coordinates_1_d(d_x, d_y, d_z) as usize] = BlockType::Grass;
            }
        }
    }

    unsafe fn generate_graphics_objects(&mut self, device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) {
        for l3 in self.terrain.values() {
            for (l2x, l2y, l2z) in L3_SIZE {
                if let Some(l2) = &l3.sub_segments[L3_SIZE.coordinates_1_d(l2x, l2y, l2z) as usize] {
    
                    for (l1x, l1y, l1z) in L2_SIZE {
                        if let Some(l1) = &l2.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z) as usize] {
                            let x_offset = l2x * L2_SIZE_BL.x + l1x * L1_SIZE_BL.x;
                            let y_offset = l2y * L2_SIZE_BL.y + l1y * L1_SIZE_BL.y;
                            let z_offset = l2z * L2_SIZE_BL.z + l1z * L1_SIZE_BL.z;
    
                            let o = l1.object(device, device_memory_properties, Vec3::new(x_offset as f32, y_offset as f32, z_offset as f32));
                            self.objects.push(o);
                        }
                    }
                }
            }
        }

    }

    unsafe fn populate(&mut self, device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) {
        let now = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).expect("time went backwards");
        let t = (now.as_millis() % 10000) as f64;
        let noise = Perlin::new(self.seed);
        for x in 0 .. L2_SIZE_BL.x as i64 {
            for z in 0 .. L2_SIZE_BL.z as i64 {
                let y = (40. * noise.get([t, (x as f64) / 150., (z as f64) / 150.])).floor().max(0.);
                self.set_block(ICoords {x, y: y as i64, z}, BlockType::Grass);
            }
        }


        for (l2x, l2y, l2z) in L3_SIZE {
            if let Some(l2) = &self.terrain[&(0, 0, 0)].sub_segments[L3_SIZE.coordinates_1_d(l2x, l2y, l2z) as usize] {

                for (l1x, l1y, l1z) in L2_SIZE {
                    if let Some(l1) = &l2.sub_segments[L2_SIZE.coordinates_1_d(l1x, l1y, l1z) as usize] {
                        let x_offset = l2x * L2_SIZE_BL.x + l1x * L1_SIZE_BL.x;
                        let y_offset = l2y * L2_SIZE_BL.y + l1y * L1_SIZE_BL.y;
                        let z_offset = l2z * L2_SIZE_BL.z + l1z * L1_SIZE_BL.z;

                        let o = l1.object(device, device_memory_properties, Vec3::new(x_offset as f32, y_offset as f32, z_offset as f32));
                        self.objects.push(o);
                    }
                }
            }
        }

        println!("world has {} objects.", self.objects.len());
    }

    pub fn get_block(self, coords: ICoords) -> BlockType {
        let (l3x, l2x, l1x) = coords.decompose(Axis::X);
        let (l3y, l2y, l1y) = coords.decompose(Axis::Y);
        let (l3z, l2z, l1z) = coords.decompose(Axis::Z);

        let l3coords = L3_SIZE.coordinates_1_d(l3x as u64, l3y as u64, l3z as u64) as usize;
        let l2coords = L2_SIZE.coordinates_1_d(l2x as u64, l2y as u64, l2z as u64) as usize;
        let l1coords = L1_SIZE.coordinates_1_d(l1x as u64, l1y as u64, l1z as u64) as usize;

        if let Some(l2) = &self.terrain[&(0, 0, 0)].sub_segments[l3coords] {
            if let Some(l1) = &l2.sub_segments[l2coords] {
                return l1.blocks[l1coords];
            }
        }
        BlockType::NoBlock
    }

    pub fn set_block(&mut self, coords: ICoords, block: BlockType) {
        if coords.x >= L3_SIZE_BL.x as i64 || coords.y >= L3_SIZE_BL.y as i64 || coords.z >= L3_SIZE_BL.z as i64 {
            println!("[set_block]: coordinates ({}, {}, {}) are out of bounds", coords.x, coords.y, coords.z);
            return;
        }

        let (l3x, l2x, l1x) = coords.decompose(Axis::X);
        let (l3y, l2y, l1y) = coords.decompose(Axis::Y);
        let (l3z, l2z, l1z) = coords.decompose(Axis::Z);

        let l3coords = L3_SIZE.coordinates_1_d(l3x as u64, l3y as u64, l3z as u64) as usize;
        let l2coords = L2_SIZE.coordinates_1_d(l2x as u64, l2y as u64, l2z as u64) as usize;
        let l1coords = L1_SIZE.coordinates_1_d(l1x as u64, l1y as u64, l1z as u64) as usize;

        let l2 = self.terrain.get_mut(&(0, 0, 0)).unwrap().sub_segments[l3coords].get_or_insert_with(L2Segment::default);
        let l1 = l2.sub_segments[l2coords].get_or_insert_with(L1Segment::default);
        l1.blocks[l1coords] = block;
    }
}