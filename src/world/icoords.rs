use std::ops::{Add, Mul, Div, Rem};

use super::Axis;
use super::*;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl Mul for ICoords {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        ICoords { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl Div for ICoords {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        ICoords { x: self.x.div_euclid(rhs.x), y: self.y.div_euclid(rhs.y), z: self.z.div_euclid(rhs.z) }
    }
}

impl Rem for ICoords {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        ICoords { x: self.x.rem_euclid(rhs.x), y: self.y.rem_euclid(rhs.y), z: self.z.rem_euclid(rhs.z) }
    }
}

impl ICoords {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub fn triple(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    /// for a given world coordinate, find the coordinates of the respective L3, L2 and L1 segments that contain this coordinate
    /// - `along`: the axis that should be decomposed
    pub fn decompose(&self, along: Axis) -> (i64, i64, i64) {
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

    /// global coordinates of the L3 segment that contains self
    pub fn l3_glob(&self) -> ICoords {
        let x = self.x.div_euclid(L3_SIZE_BL.x as i64);
        let y = self.y.div_euclid(L3_SIZE_BL.y as i64);
        let z = self.z.div_euclid(L3_SIZE_BL.z as i64);
        ICoords {x, y, z}
    }

    /// global coordinates of the L2 segment that contains self
    pub fn l2_glob(&self) -> ICoords {
        let x = self.x.div_euclid(L2_SIZE_BL.x as i64);
        let y = self.y.div_euclid(L2_SIZE_BL.y as i64);
        let z = self.z.div_euclid(L2_SIZE_BL.z as i64);
        ICoords {x, y, z}
    }

    /// local coordinates of the L2 segment (within its parent L3) that contains self
    pub fn l2_loc(&self) -> ICoords {
        let x = self.x.div_euclid(L2_SIZE_BL.x as i64).rem_euclid(L3_SIZE.x as i64);
        let y = self.y.div_euclid(L2_SIZE_BL.y as i64).rem_euclid(L3_SIZE.y as i64);
        let z = self.z.div_euclid(L2_SIZE_BL.z as i64).rem_euclid(L3_SIZE.z as i64);
        ICoords {x, y, z}
    }

    /// global coordinates of the L1 segment that contains self
    pub fn l1_glob(&self) -> ICoords {
        let x = self.x.div_euclid(L1_SIZE_BL.x as i64);
        let y = self.y.div_euclid(L1_SIZE_BL.y as i64);
        let z = self.z.div_euclid(L1_SIZE_BL.z as i64);
        ICoords {x, y, z}
    }

    /// local coordinates of the L1 segment (within its parent L2) that contains self
    pub fn l1_loc(&self) -> ICoords {
        let x = self.x.div_euclid(L1_SIZE_BL.x as i64).rem_euclid(L2_SIZE.x as i64);
        let y = self.y.div_euclid(L1_SIZE_BL.y as i64).rem_euclid(L2_SIZE.y as i64);
        let z = self.z.div_euclid(L1_SIZE_BL.z as i64).rem_euclid(L2_SIZE.z as i64);
        ICoords {x, y, z}
    }

    /// local coordinates of self within the L1 segment
    pub fn bl_loc(&self) -> ICoords {
        let x = self.x.rem_euclid(L1_SIZE_BL.x as i64);
        let y = self.y.rem_euclid(L1_SIZE_BL.y as i64);
        let z = self.z.rem_euclid(L1_SIZE_BL.z as i64);
        ICoords {x, y, z}
    }
}
