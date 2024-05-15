
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum BlockType {
    NoBlock,
    Grass
}

impl Default for BlockType {
    fn default() -> Self {
        return Self::NoBlock;
    }
}

impl BlockType {
    pub fn color(&self) -> Option<[f32; 4]> {
        match &self {
            BlockType::Grass => Some([0.0, 1.0, 0.0, 1.0]),
            _ => Some([1.0, 1.0, 1.0, 1.0])
        }
    }

    pub fn is_solid(&self) -> bool {
        *self != BlockType::NoBlock
    }
}
