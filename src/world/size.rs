use std::iter::Iterator;
use crate::world::ICoords;

pub struct Size3DIterator {
    size: Size3D,
    i: u64
}

impl Iterator for Size3DIterator {
    type Item = (u64, u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.size.volume() {
            return None
        }
        let z = self.i % self.size.z;
        let y = (self.i / self.size.z) % self.size.y;
        let x = (self.i / (self.size.z * self.size.y)) % self.size.x;

        self.i += 1;

        Some((x, y, z))
    }
}

#[derive(Clone, Copy)]
pub struct Size3D {
    pub x: u64,
    pub y: u64,
    pub z: u64
}

impl IntoIterator for Size3D {
    type Item = (u64, u64, u64);
    type IntoIter = Size3DIterator;

    fn into_iter(self) -> Self::IntoIter {
        Size3DIterator { size: self, i: 0 }
    }
}

impl Size3D {
    pub fn volume(&self) -> u64 {
        self.x * self.y * self.z
    }

    pub fn num_vertices(&self) -> u64 {
        (self.x + 1) * (self.y + 1) * (self.z + 1)
    }

    pub fn coordinates_1_d(&self, x: u64, y: u64, z: u64) -> u64 {
        self.y * self.z * x + self.z * y + z
    }

    pub fn vertex_coordinates_1_d(&self, x: u64, y: u64, z: u64) -> u64 {
        (self.y + 1) * (self.z + 1) * x + (self.z + 1) * y + z
    }

    pub fn contains(&self, c: ICoords) -> bool {
        (0..self.x as i64).contains(&c.x) 
        && (0..self.y as i64).contains(&c.y) 
        && (0..self.z as i64).contains(&c.z)
    }
}
