use crate::world::*;
use crate::world::block::*;
use rand::prelude::*;

pub struct BlockObject {
    position: Vec3,
    size: Size3D,
    blocks: Vec<BlockType>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub is_ticking: bool,
}

impl BlockObject {
    pub fn new(size: Size3D, position: Vec3, blocks: Vec<BlockType>) -> Self {
        BlockObject {
            position,
            size,
            blocks,
            vertices: Vec::new(),
            indices: Vec::new(),
            is_ticking: false,
        }
    }

    pub fn tick(&mut self, t: u128) {
        self.position.y = 10. + ((((t % 10000)) as f32) / 100.).sin();
    }

    pub fn update_indices(&mut self) {
        let block_list = self.block_list();
        self.indices = vec![0; 36 * block_list.len()];
        for (j, &(x, y, z)) in block_list.iter().enumerate() {
            if self.blocks[self.size.coordinates_1_d(x, y, z)] == BlockType::Grass {
                for i in 0 .. 36 {
                    self.indices[j * 36 + i] = self.size.coordinates_1_d(x, y, z) as u32 * 8 + BLOCK_TRIANGLE_INDICES[i] as u32;
                }
            }
        }
    }

    pub fn update_vertices(&mut self) {
        self.vertices = vec![Vertex::default(); self.size.volume() * 8];
        let mut rng = thread_rng();

        for (i, (x, y, z)) in self.size.into_iter().enumerate() {
            let color: [f32; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            // let color: [f32; 4] = [1., 1., 0., 0.8];

            for (j, [dx, dy, dz]) in BLOCK_VERTICES.iter().enumerate() {
                let vertex = Vertex {
                    pos: [(x + dx) as f32 + self.position.x, (y + dy) as f32 + self.position.y, (z + dz) as f32 + self.position.z, 1.0],
                    color,
                    tex_coord: [0., 0.]
                };
                self.vertices[i * 8 + j] = vertex;
            }
        }
    }

    fn block_list(&self) -> Vec<(usize, usize, usize)> {
        let mut list = vec![(0, 0, 0); 0];
        for (x, y, z) in self.size {
            if self.blocks[self.size.coordinates_1_d(x, y, z)] != BlockType::NoBlock {
                list.push((x, y, z));
            }
        }
        return list;
    }

    pub fn num_blocks(&self) -> u32 {
        let mut n = 0;
        for block in self.blocks.iter() {
            if block != &BlockType::NoBlock {
                n += 1;
            }
        }
        return n;
    }
}

impl TriangleGraphicsObject for BlockObject {
    fn vertices(&self) -> &Vec<Vertex> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}
