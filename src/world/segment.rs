use glam::Vec3;
use crate::world::*;
use crate::world::size::*;


#[derive(Clone)]
pub struct L1Segment {
    pub blocks: Vec<BlockType>,
}

impl L1Segment {
    pub fn object(&self, pos: Vec3) -> BlockObject {
        let mut o = BlockObject::new(L1_SIZE, pos);
        o.blocks = self.blocks.to_vec();
        o.update_indices();
        o.update_vertices();
        return o;
    }
}

impl Default for L1Segment {
    fn default() -> Self {
        let mut blocks = Vec::new();
        blocks.resize(L1_SIZE.volume(), BlockType::NoBlock);
        L1Segment {
            blocks: blocks,
        }
    }
}

impl L1Segment {
    pub fn enumerate_blocks(&self) -> Vec<(usize, usize, usize, BlockType)> {
        let mut ret: Vec<(usize, usize, usize, BlockType)> = Vec::new();
        for (x, y, z) in L1_SIZE {
            ret.push((x, y, z, self.blocks[L1_SIZE.coordinates_1_d(x, y, z)]))
        }
        ret
    }
}

#[derive(Clone)]
pub struct L2Segment {
    pub sub_segments: Vec<Option<L1Segment>>,
}

impl Default for L2Segment {
    fn default() -> Self {
        let mut sub_segments = Vec::new();
        sub_segments.resize(L2_SIZE.volume(), None);
        L2Segment {
            sub_segments: sub_segments,
        }
    }
}

#[derive(Clone)]
pub struct L3Segment {
    pub sub_segments: Vec<Option<L2Segment>>,
}

impl Default for L3Segment {
    fn default() -> Self {
        let mut sub_segments = Vec::new();
        sub_segments.resize(L3_SIZE.volume(), None);
        L3Segment {
            sub_segments: sub_segments,
        }
    }
}

// 'a is a lifetime specifier
// it defines how the lifetimes of the world and sub_segments relate
pub struct L4Segment {
    pub sub_segments: Vec<Option<L3Segment>>,
}

impl Default for L4Segment {
    fn default() -> Self {
        let mut sub_segments = Vec::new();
        sub_segments.resize(L4_SIZE.volume(), None);
        L4Segment {
            sub_segments: sub_segments,
        }
    }
}
