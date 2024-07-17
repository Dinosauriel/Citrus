use crate::world::{icoords::ICoords, size::Size3D};
use super::camera::Camera;

// set of l1segments that should be rendered based on camera
pub fn scan(camera: &Camera) -> Vec<ICoords> {
    let c0 = ICoords::new(camera.ray.origin.x as i64, camera.ray.origin.y as i64, camera.ray.origin.z as i64).l1_glob();
    let diameter: i64 = 5;
    let r = diameter / 2;
    let cube = Size3D {x: diameter as u64, y: diameter as u64, z: diameter as u64};
    let segs = cube.into_iter().map(|delta| {c0 + delta + ICoords::new(-r, -r, -r)}).collect();
   segs
}
