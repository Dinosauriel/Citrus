use std::iter::Iterator;
use crate::world::ICoords;

pub struct Size3DIterator {
    size: Size3D,
    i: u64
}

impl Iterator for Size3DIterator {
    type Item = ICoords;

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
    type Item = ICoords;
    type IntoIter = Size3DIterator;

    fn into_iter(self) -> Self::IntoIter {
        Size3DIterator { size: self, i: 0 }
    }
}

impl Into<ICoords> for Size3D {
    fn into(self) -> ICoords {
        ICoords {x: self.x as i64, y: self.y as i64, z: self.z as i64}
    }
}

impl Size3D {
    pub fn volume(&self) -> u64 {
        self.x * self.y * self.z
    }

    /// maps 3d coordinates to a 1d index
    pub fn c1d(&self, coords: ICoords) -> u64 {
        self.y * self.z * coords.x as u64 + self.z * coords.y as u64 + coords.z as u64
    }

    pub fn contains(&self, c: ICoords) -> bool {
        (0..self.x as i64).contains(&c.x) 
        && (0..self.y as i64).contains(&c.y) 
        && (0..self.z as i64).contains(&c.z)
    }

    /// maps 1d coordinate to 3d indices
    pub fn c3d(&self, i: u64) -> ICoords {
        let x = (i / (self.y * self.z)) % self.x;
        let y = (i / self.z) % self.y;
        let z = i % self.z;
        ICoords {x: x as i64, y: y as i64, z: z as i64}
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
    pub fn c1d(&self, x: i64, y: i64) -> usize {
        (self.y * x as u64 + y as u64) as usize
    }
}

pub struct Size2DIterator {
    size: Size2D,
    i: u64
}

impl Iterator for Size2DIterator {
    type Item = (i64, i64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.size.area() {
            return None
        }
        let y = self.i % self.size.y;
        let x = (self.i / self.size.y) % self.size.x;

        self.i += 1;

        Some((x as i64, y as i64))
    }
}

impl IntoIterator for Size2D {
    type Item = (i64, i64);
    type IntoIter = Size2DIterator;

    fn into_iter(self) -> Self::IntoIter {
        Size2DIterator { size: self, i: 0 }
    }
}
