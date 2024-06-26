use crate::world::*;
use crate::world::size::*;

pub const L1_SIZE_BITS: u64 = 5;
pub const L1_SIZE: Size3D = Size3D { x: 1 << L1_SIZE_BITS, y: 1 << L1_SIZE_BITS, z: 1 << L1_SIZE_BITS };
pub const L1_SIZE_BL: Size3D = L1_SIZE;

#[derive(Clone, Debug)]
pub struct L1Segment {
    pub blocks: Vec<BlockType>,
}

impl L1Segment {
    pub fn number_of_solid_blocks(&self) -> usize {
        self.blocks.iter().filter(|&t| t != &BlockType::NoBlock).collect::<Vec<_>>().len()
    }
}

impl Default for L1Segment {
    fn default() -> Self {
        let mut blocks = Vec::new();
        blocks.resize(L1_SIZE.volume() as usize, BlockType::NoBlock);
        L1Segment {
            blocks,
        }
    }
}

pub const L2_SIZE_BITS: u64 = 3;
pub const L2_SIZE: Size3D = Size3D { x: 1 << L2_SIZE_BITS, y: 1 << L2_SIZE_BITS, z: 1 << L2_SIZE_BITS };
pub const L2_SIZE_BL: Size3D = Size3D { x: L1_SIZE_BL.x * L2_SIZE.x, y: L1_SIZE_BL.y * L2_SIZE.y, z: L1_SIZE_BL.z * L2_SIZE.z };

#[derive(Clone)]
pub struct L2Segment {
    pub sub_segments: Vec<Option<L1Segment>>,
}

impl L2Segment {
    pub fn number_of_l1_segments(&self) -> usize {
        self.sub_segments.iter().filter(|s| s.is_some()).collect::<Vec<_>>().len()
    }
}

impl Default for L2Segment {
    fn default() -> Self {
        let mut sub_segments = Vec::new();
        sub_segments.resize(L2_SIZE.volume() as usize, None);
        L2Segment {
            sub_segments,
        }
    }
}

pub const L3_SIZE_BITS: u64 = 3;
pub const L3_SIZE: Size3D = Size3D { x: 1 << L3_SIZE_BITS, y: 1 << L3_SIZE_BITS, z: 1 << L3_SIZE_BITS };
pub const L3_SIZE_BL: Size3D = Size3D { x: L2_SIZE_BL.x * L3_SIZE.x, y: L2_SIZE_BL.y * L3_SIZE.y, z: L2_SIZE_BL.z * L3_SIZE.z };

#[derive(Clone)]
pub struct L3Segment {
    pub sub_segments: Vec<Option<L2Segment>>,
}

impl L3Segment {
    pub fn number_of_l2_segments(&self) -> usize {
        self.sub_segments.iter().filter(|s| s.is_some()).collect::<Vec<_>>().len()
    }
}

impl Default for L3Segment {
    fn default() -> Self {
        let mut sub_segments = Vec::new();
        sub_segments.resize(L3_SIZE.volume() as usize, None);
        L3Segment {
            sub_segments,
        }
    }
}
