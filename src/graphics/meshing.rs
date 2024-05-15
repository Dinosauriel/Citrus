
use std::usize;

use glam::Vec3;
use rand::{thread_rng, Rng};

use crate::{graphics::vertex::ColoredVertex, world::{
    segment::{L1Segment, L1_SIZE_BL}, size::Size2D, Face, BL_VERTICES
}};

/// returns a `L1_SIZE_BL.x * L1_SIZE_BL.y` vector of u32 integers `a` which each encode what blocks are solid
///
/// each bit in `a` encodes whether or not a block is solid (1) or not (0)
/// 
/// example `a` in big endian order:
/// ```
/// high z        low z
/// |                 |
/// 1 0 0 0 ... 0 1 1 0
/// <---- 32 bits ---->
/// ```
fn l1_solids(seg: &L1Segment) -> Vec<u32> {
    seg.blocks.chunks(L1_SIZE_BL.z as usize)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u32, |acc, (i, block)| { acc | ((block.is_solid() as u32) << i) })
        }).collect()
}

/// - `neighbours`: [XPos, XNeg, YPos, YNeg, ZPos, ZNeg]
pub fn mesh_l1_segment(seg: &L1Segment, neighbours: &[Option<L1Segment>; 6], position: Vec3) -> (Vec<ColoredVertex>, Vec<u32>) {
    let solids = l1_solids(seg);
    let plane_size = Size2D { x: L1_SIZE_BL.x, y: L1_SIZE_BL.y };

    // bit x, y, z indicates whether or not voxel x, y, z is exposed in what direction
    let mut faces = vec![solids.clone(); Face::all().len()];

    for (x, y) in plane_size {
        faces[Face::ZPos as usize][plane_size.c1d(x, y)] &= !(solids[plane_size.c1d(x, y)] >> 1);
        faces[Face::ZNeg as usize][plane_size.c1d(x, y)] &= !(solids[plane_size.c1d(x, y)] << 1);

        if x > 0 {
            faces[Face::XNeg as usize][plane_size.c1d(x, y)] &= !solids[plane_size.c1d(x - 1, y)];
        }
        if x < L1_SIZE_BL.x - 1 {
            faces[Face::XPos as usize][plane_size.c1d(x, y)] &= !solids[plane_size.c1d(x + 1, y)];
        }

        if y > 0 {
            faces[Face::YNeg as usize][plane_size.c1d(x, y)] &= !solids[plane_size.c1d(x, y - 1)];
        }
        if y < L1_SIZE_BL.y - 1 {
            faces[Face::YPos as usize][plane_size.c1d(x, y)] &= !solids[plane_size.c1d(x, y + 1)];
        }
    }

    // now we know which faces are exposed
    let mut indices = Vec::<u32>::new();
    for (x, y, z) in L1_SIZE_BL {
        let v = 8 * L1_SIZE_BL.c1d(x, y, z);
        for face in Face::all() {
            if faces[face as usize][plane_size.c1d(x, y)] & (1u32 << z) != 0 {
                indices.extend_from_slice(&face.indices().map(|idx| {v as u32 + idx}));
            }
        }
    }

    // check which vertices are actually used
    let mut vertex_used = vec![false; 8 * L1_SIZE_BL.volume() as usize];
    for &index in &indices {
        vertex_used[index as usize] = true;
    }

    let mut permutation = vec![0 as usize; vertex_used.len()];
    let mut n_vertices_used = 0;
    for (i, &used) in vertex_used.iter().enumerate() {
        if used {
            permutation[i] = n_vertices_used;
            n_vertices_used += 1;
        }
    }

    // println!("{} of {} vertices used", n_vertices_used, vertex_used.len());

    let mut vertices = vec![ColoredVertex::default(); n_vertices_used];
    let mut rng = thread_rng();

    for (x, y, z) in L1_SIZE_BL {
        let v = 8 * L1_SIZE_BL.c1d(x, y, z);
        let color = [rng.gen(), rng.gen(), rng.gen(), 0.8];
        for (corner, [dx, dy, dz]) in BL_VERTICES.iter().enumerate() {
            if vertex_used[v as usize + corner] {

                vertices[permutation[v as usize + corner]] = ColoredVertex {
                    pos: [
                        (x + dx) as f32 + position.x,
                        (y + dy) as f32 + position.y,
                        (z + dz) as f32 + position.z,
                        1.0
                    ],
                    color,
                };
            }
        }
    }

    // translate indices
    for i in 0..indices.len() {
        indices[i] = permutation[indices[i] as usize] as u32;
    }

    (vertices, indices)
}
