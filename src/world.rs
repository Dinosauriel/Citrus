pub mod segment;
pub mod icoords;
pub mod object;
pub mod ray;
pub mod size;
pub mod block;

use std::collections::HashMap;
use noise::{NoiseFn, Simplex};
use glam::Vec3;
use crate::graphics::meshing;
use crate::profiler::*;
use object::*;
use size::*;
use segment::*;
use block::*;
use icoords::*;

// the indices of the triangles constituting the block face facing in negative x direction
const INDICES_NEG_X: [u32; 6] = [
    1, 0, 2,
    1, 2, 3,
];

const INDICES_POS_X: [u32; 6] = [
    4, 5, 6,
    5, 7, 6,
];

const INDICES_NEG_Y: [u32; 6] = [
    0, 1, 4,
    1, 5, 4,
];

const INDICES_POS_Y: [u32; 6] = [
    2, 6, 3,
    3, 6, 7,
];

const INDICES_NEG_Z: [u32; 6] = [
    0, 4, 2,
    2, 4, 6,
];

const INDICES_POS_Z: [u32; 6] = [
    1, 3, 5,
    3, 7, 5,
];

pub const BL_VERTICES: [[u64; 3]; 8] = [
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

#[derive(Debug, Clone, Copy)]
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

    pub fn indices(&self) -> [u32; 6] {
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

pub struct World<'a> {
    pub objects: Vec<RawObject<'a>>,
    pub terrain: HashMap<ICoords, L3Segment>,
    seed: u32,
}

impl<'a> World<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) -> Self {
        let mut w = World {
            objects: Vec::new(),
            terrain: HashMap::new(),
            seed: 12,
        };

        for l1c in (Size3D {x: 4, y: 4, z: 4}) {
        // for (l1x, l1y, l1z) in [(0, 0, 0)] {
            w.generate_l1_segment(l1c * L1_SIZE_BL.into());
        }

        w.generate_graphics_objects(device, device_memory_properties);
        w
    }

    fn l3_segment(&self, coords: ICoords) -> Option<&L3Segment> {
        self.terrain.get(&&coords.l3_glob())
    }

    fn create_or_get_l3(&mut self, coords: ICoords) -> &mut L3Segment {
        if !self.terrain.contains_key(&coords.l3_glob()) {
            self.terrain.insert(coords.l3_glob(), L3Segment::default());
        }
        self.terrain.get_mut(&coords.l3_glob()).unwrap()
    }

    fn l2_segment(&self, coords: ICoords) -> Option<&L2Segment> {
        if let Some(l3_seg) = self.l3_segment(coords) {
            let l2c = coords.l2_loc();
            return l3_seg.sub_segments[L3_SIZE.c1d(l2c) as usize].as_ref();
        }

        None
    }

    fn create_or_get_l2(&mut self, coords: ICoords) -> &mut L2Segment {
        let l3_seg = self.create_or_get_l3(coords);
        let l2c = coords.l2_loc();
        if l3_seg.sub_segments[L3_SIZE.c1d(l2c) as usize].is_none() {
            l3_seg.sub_segments[L3_SIZE.c1d(l2c) as usize] = Some(L2Segment::default());
        }

        l3_seg.sub_segments[L3_SIZE.c1d(l2c) as usize].as_mut().unwrap()
    }

    fn l1_segment(&self, coords: ICoords) -> Option<&L1Segment> {
        if let Some(l2_seg) = self.l2_segment(coords) {
            let l1_coords = coords.l1_loc();
            return l2_seg.sub_segments[L2_SIZE.c1d(l1_coords) as usize].as_ref();
        }

        None
    }

    fn create_or_get_l1(&mut self, coords: ICoords) -> &mut L1Segment {
        let l2_seg = self.create_or_get_l2(coords);
        let l1_coords = coords.l1_loc();
        if l2_seg.sub_segments[L2_SIZE.c1d(l1_coords) as usize].is_none() {
            l2_seg.sub_segments[L2_SIZE.c1d(l1_coords) as usize] = Some(L1Segment::default());
        }

        l2_seg.sub_segments[L2_SIZE.c1d(l1_coords) as usize].as_mut().unwrap()
    }

    /// * `coords` - coordinates of the 0 0 0 block in the desired l1_segment
    fn generate_l1_segment(&mut self, coords: ICoords) {
        p_start("generate_l1_segment");
        let noise = Simplex::new(self.seed);
        let l1_seg = self.create_or_get_l1(coords);

        for delta in L1_SIZE_BL {
            p_start("noise.get");
            let v = noise.get([
                (coords.x + delta.x) as f64 / 50.,
                (coords.y + delta.y) as f64 / 50.,
                (coords.z + delta.z) as f64 / 50.]);
            p_end("noise.get");
            if v > 0. {
                l1_seg.blocks[L1_SIZE_BL.c1d(delta) as usize] = BlockType::Grass;
            }
        }
        p_end("generate_l1_segment");
    }

    unsafe fn generate_graphics_objects(&mut self, device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) {
        for l3 in self.terrain.values() {
            for l2c in L3_SIZE {
                if let Some(l2) = &l3.sub_segments[L3_SIZE.c1d(l2c) as usize] {
                    for l1c in L2_SIZE {
                        if let Some(l1) = &l2.sub_segments[L2_SIZE.c1d(l1c) as usize] {
                            let offset = l2c * L2_SIZE_BL.into() + l1c * L1_SIZE_BL.into();

                            p_start("mesh_l1_segment");
                            let (vertices, indices) = meshing::mesh_l1_segment(l1, 
                                [
                                    self.l1_segment((l1c + ICoords::new(1, 0, 0)) * L1_SIZE.into()),
                                    self.l1_segment((l1c + ICoords::new(-1, 0, 0)) * L1_SIZE.into()),
                                    self.l1_segment((l1c + ICoords::new(0, 1, 0)) * L1_SIZE.into()),
                                    self.l1_segment((l1c + ICoords::new(0, -1, 0)) * L1_SIZE.into()),
                                    self.l1_segment((l1c + ICoords::new(0, 0, 1)) * L1_SIZE.into()),
                                    self.l1_segment((l1c + ICoords::new(0, 0, -1)) * L1_SIZE.into())], 
                                Vec3::new(offset.x as f32, offset.y as f32, offset.z as f32));
                            p_end("mesh_l1_segment");

                            let o = RawObject::new(device, device_memory_properties, &vertices, &indices);
                            self.objects.push(o);
                        }
                    }
                }
            }
        }
    }

    pub fn get_block(self, coords: ICoords) -> BlockType {
        let l2c = L2_SIZE.c1d(coords.l2_loc()) as usize;
        let l1c = L1_SIZE.c1d(coords.l1_loc()) as usize;
        let blc = L3_SIZE.c1d(coords.bl_loc()) as usize;

        if let Some(l2) = &self.terrain[&coords.l3_glob()].sub_segments[l2c] {
            if let Some(l1) = &l2.sub_segments[l1c] {
                return l1.blocks[blc];
            }
        }
        BlockType::NoBlock
    }

    pub fn set_block(&mut self, coords: ICoords, block: BlockType) {
        if coords.x >= L3_SIZE_BL.x as i64 || coords.y >= L3_SIZE_BL.y as i64 || coords.z >= L3_SIZE_BL.z as i64 {
            println!("[set_block]: coordinates ({}, {}, {}) are out of bounds", coords.x, coords.y, coords.z);
            return;
        }

        let l2c = L2_SIZE.c1d(coords.l2_loc()) as usize;
        let l1c = L1_SIZE.c1d(coords.l1_loc()) as usize;
        let blc = L3_SIZE.c1d(coords.bl_loc()) as usize;

        let l2 = self.terrain.get_mut(&coords.l3_glob()).unwrap().sub_segments[l2c].get_or_insert_with(L2Segment::default);
        let l1 = l2.sub_segments[l1c].get_or_insert_with(L1Segment::default);
        l1.blocks[blc] = block;
    }
}