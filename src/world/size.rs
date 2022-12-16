#[derive(Clone, Copy)]
pub struct Size3D {
    pub x: usize,
    pub y: usize,
    pub z: usize
}

impl Size3D {
    pub fn volume(&self) -> usize {
        return self.x * self.y * self.z;
    }

    pub fn num_vertices(&self) -> usize {
        return (self.x + 1) * (self.y + 1) * (self.z + 1);
    }

    pub fn coordinates_1_d(&self, x: usize, y: usize, z: usize) -> usize {
        return self.x * self.z * y + self.z * x + z;
    }

    pub fn vertex_coordinates_1_d(&self, x: usize, y: usize, z: usize) -> usize {
        return (self.x + 1) * (self.z + 1) * y + (self.z + 1) * x + z;
    }
}

pub const L1_SEGMENT_SIZE: Size3D = Size3D { x: 8, y: 8, z: 8 };
pub const L2_SEGMENT_SIZE: Size3D = Size3D { x: 8, y: 8, z: 8 };
pub const L3_SEGMENT_SIZE: Size3D = Size3D { x: 8, y: 8, z: 8 };
pub const L4_SEGMENT_SIZE: Size3D = Size3D { x: 8, y: 8, z: 8 };
