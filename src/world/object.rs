use crate::world::*;
use crate::world::block::*;

pub struct BlockObject {
    position: Vec3,
    size: Size3D,
    blocks: Vec<BlockType>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl BlockObject {
    pub fn new(size: Size3D, position: Vec3, blocks: Vec<BlockType>) -> Self {
        BlockObject {
            position,
            size,
            blocks,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn update_indices(&mut self) {
        let block_list = self.block_list();
        self.indices = vec![0; 36 * block_list.len()];
        for (j, &(x, y, z)) in block_list.iter().enumerate() {
            if self.blocks[self.size.coordinates_1_d(x, y, z)] == BlockType::Grass {
                for i in 0 .. 36 {
                    let [d_x, d_y, d_z] = BLOCK_VERTICES[BLOCK_TRIANGLE_INDICES[i]];
                    self.indices[j * 36 + i] = self.size.vertex_coordinates_1_d(x + d_x, y + d_y, z + d_z) as u32;
                }
            }
        }
    }

    pub fn update_vertices(&mut self) {
        self.vertices = vec![Vertex::default(); self.size.num_vertices()];
        for x in 0 .. self.size.x + 1 {
            for y in 0 .. self.size.y + 1 {
                for z in 0 .. self.size.z + 1 {
                    let vertex = Vertex {
                        pos: [x as f32 + self.position.x, y as f32 + self.position.y, z as f32 + self.position.z, 1.0],
                        color: [1.0, 1.0, 0.0, 1.0]
                    };
                    self.vertices[self.size.vertex_coordinates_1_d(x, y, z)] = vertex;
                }
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
