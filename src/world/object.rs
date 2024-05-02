use crate::graphics::buffer::Buffer;
use crate::graphics::vertex::ColoredVertex;
use crate::world::*;
use crate::world::block::*;
use rand::prelude::*;

pub struct BlockObject<'a> {
    position: Vec3,
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<u32>,
    vertex_buffer: Buffer<'a>,
    index_buffer: Buffer<'a>,

    pub is_ticking: bool,
}

impl<'a> BlockObject<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
            size: &Size3D, position: &Vec3, blocks: &Vec<BlockType>) -> Self {

        let block_list = Self::enlist_blocks(&blocks, &size);
        let vertices = Self::vertices_from_block_list(&block_list, &position);
        let indices = Self::indices_from_block_list(&block_list, &size, &blocks);

        let vertex_buffer = Buffer::new_vertex::<ColoredVertex>(vertices.len(), device, device_memory_properties);
        vertex_buffer.fill(&vertices);
        let index_buffer = Buffer::new_index(indices.len(), device, device_memory_properties);
        index_buffer.fill(&indices);

        BlockObject {
            position: *position,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            is_ticking: false,
        }
    }

    pub fn tick(&mut self, t: u128) {
        self.position.y = 10. + (((t % 10000) as f32) / 500.).sin();
        self.position.x = 10. + (((t % 10000) as f32) / 500.).cos();
    }

    fn indices_from_block_list(block_list: &Vec<(u64, u64, u64)>, size: &Size3D, blocks: &Vec<BlockType>) -> Vec<u32> {
        let mut indices = vec![];

        for (i, (x, y, z)) in block_list.into_iter().enumerate() {
            let c = ICoords {x: *x as i64, y: *y as i64, z: *z as i64};
            for face in Face::all() {
                // coordinates of the neighbouring block
                let c_p = c + face.numeric();
                if size.contains(c_p) 
                    && blocks[size.coordinates_1_d(c_p.x as u64, c_p.y as u64, c_p.z as u64) as usize] != BlockType::NoBlock {
                        // skip these indices if the neighbouring coordinates are not empty
                        continue;
                }

                for index in face.indices() {
                    indices.push((i * 8 + index as usize) as u32)
                }
            }
        }
        indices
    }

    fn vertices_from_block_list(block_list: &Vec<(u64, u64, u64)>, position: &Vec3) -> Vec<ColoredVertex> {
        let mut vertices = vec![ColoredVertex::default(); block_list.len() * 8];
        let mut rng = thread_rng();

        for (i, (x, y, z)) in block_list.into_iter().enumerate() {
            let color: [f32; 4] = [rng.gen(), rng.gen(), rng.gen(), 0.8];
            // let color: [f32; 4] = [1., 1., 0., 0.8];

            for (j, [dx, dy, dz]) in BL_VERTICES.iter().enumerate() {
                let vertex = ColoredVertex {
                    pos: [
                        (x + dx) as f32 + position.x,
                        (y + dy) as f32 + position.y,
                        (z + dz) as f32 + position.z,
                        1.0
                    ],
                    color,
                };
                vertices[i * 8 + j] = vertex;
            }
        }
        vertices
    }

    // a list of coordinates of blocks that are not NoBlock
    fn enlist_blocks(blocks: &Vec<BlockType>, size: &Size3D) -> Vec<(u64, u64, u64)> {
        size.into_iter().filter(|(x, y, z)| blocks[size.coordinates_1_d(*x, *y, *z) as usize] != BlockType::NoBlock).collect()
    }
}

impl<'a> GraphicsObject<'a, ColoredVertex> for BlockObject<'a> {
    fn index_buffer(&self) -> &Buffer<'a> {
        &self.index_buffer
    }

    fn vertex_buffer(&self) -> &Buffer<'a> {
        &self.vertex_buffer
    }

    fn vertices(&self) -> &Vec<ColoredVertex> {
        &self.vertices
    }

    fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}

pub struct TerrainObject<'a> {
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<u32>,
    vertex_buffer: Buffer<'a>,
    index_buffer: Buffer<'a>,
}

impl<'a> TerrainObject<'a> {
    // pub unsafe fn new(g_state: &'a GraphicState, segment: &L1Segment, position: Vec3) -> Self {
    //     let block_list = enlist_blocks(&segment.blocks, &L1_SIZE_BL);
    //     let vertices = vertices_from_block_list(&block_list, &position);
    //     let indices = indices_from_block_list(&block_list, &L1_SIZE_BL, &segment.blocks);

    //     let vertex_buffer = Buffer::new_vertex::<ColoredVertex>(vertices.len(), &g_state.device, &g_state.device_memory_properties);
    //     vertex_buffer.fill(&vertices);
    //     let index_buffer = Buffer::new_index(indices.len(), &g_state.device, &g_state.device_memory_properties);
    //     index_buffer.fill(&indices);

    //     TerrainObject {
    //         vertices,
    //         indices,
    //         vertex_buffer,
    //         index_buffer
    //     }
    // }
}

impl<'a> GraphicsObject<'a, ColoredVertex> for TerrainObject<'a> {
    fn index_buffer(&self) -> &Buffer<'a> {
        &self.index_buffer
    }

    fn vertex_buffer(&self) -> &Buffer<'a> {
        &self.vertex_buffer
    }

    fn vertices(&self) -> &Vec<ColoredVertex> {
        &self.vertices
    }

    fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}
