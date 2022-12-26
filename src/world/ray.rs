use glam::Vec3;
use crate::world::*;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3
}

trait AbsMin {
    fn abs_min(self, b: Self) -> Self;
}

impl AbsMin for f32 {
    fn abs_min(self, b: f32) -> f32 {
        if self.abs() < b.abs() {
            return self;
        }
        return b;
    }
}

impl Ray {
    fn round_in_dir(x: f32, direction: f32) -> f32 {
        if x == x.round() {
            // x represents an integer
            return x + direction.signum();
        }
        if direction > 0. {
            return x.ceil();
        }
        return x.floor();
    }

    // return the first intersected blocks within n units
    pub fn intersected_blocks(&self, n: usize) -> Vec<Coords> {
        let mut coords = Vec::with_capacity(n);
        let mut p = self.origin;
        while self.origin.distance_squared(p) < (n * n) as f32 {
            coords.push(Coords {x: p.x as i64, y: p.y as i64, z: p.z as i64});
            let delta_x = Self::round_in_dir(p.x, self.direction.x) - p.x;
            let delta_y = Self::round_in_dir(p.y, self.direction.y) - p.y;
            let delta_z = Self::round_in_dir(p.z, self.direction.z) - p.z;
            let r = (delta_x / self.direction.x).abs_min(delta_y / self.direction.y).abs_min(delta_z / self.direction.z);
            p += r * self.direction;
        }
        return coords;
    }
}
