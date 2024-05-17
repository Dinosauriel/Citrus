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
        self.i += 1;
        Some(self.size.c3d(self.i - 1))
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

    /// maps 3d coordinates to a 1d index
    pub fn c1d(&self, x: u64, y: u64, z: u64) -> u64 {
        self.y * self.z * x + self.z * y + z
    }

    /// maps 3d coordinates to a 1d vertex index
    pub fn vc1d(&self, x: u64, y: u64, z: u64) -> u64 {
        (self.y + 1) * (self.z + 1) * x + (self.z + 1) * y + z
    }

    pub fn contains(&self, c: ICoords) -> bool {
        (0..self.x as i64).contains(&c.x) 
        && (0..self.y as i64).contains(&c.y) 
        && (0..self.z as i64).contains(&c.z)
    }

    /// maps 1d coordinate to 3d indices
    pub fn c3d(&self, i: u64) -> (u64, u64, u64) {
        let x = (i / (self.y * self.z)) % self.x;
        let y = (i / self.z) % self.y;
        let z = i % self.z;
        (x, y, z)
    }
}

#[derive(Clone, Copy)]
pub struct Size2D {
    pub x: u64,
    pub y: u64,
}

impl Size2D {
    pub fn area(&self) -> u64 {
        self.x * self.y
    }

    /// maps 2d coordinates to a 1d index
    pub fn c1d(&self, x: u64, y: u64) -> usize {
        (self.y * x + y) as usize
    }
}

pub struct Size2DIterator {
    size: Size2D,
    i: u64
}

impl Iterator for Size2DIterator {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.size.area() {
            return None
        }
        let y = self.i % self.size.y;
        let x = (self.i / self.size.y) % self.size.x;

        self.i += 1;

        Some((x, y))
    }
}

impl IntoIterator for Size2D {
    type Item = (u64, u64);
    type IntoIter = Size2DIterator;

    fn into_iter(self) -> Self::IntoIter {
        Size2DIterator { size: self, i: 0 }
    }
}
