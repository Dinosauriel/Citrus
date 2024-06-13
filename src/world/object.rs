use crate::graphics::buffer::Buffer;
use crate::graphics::vertex::ColoredVertex;
use crate::world::*;
use crate::world::block::*;
use rand::prelude::*;

pub struct RawObject<'a> {
    pub vertex_buffer: Buffer<'a>,
    pub index_buffer: Buffer<'a>,
    pub index_count: u32,
}

impl<'a> RawObject<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
                        vertices: &Vec<ColoredVertex>, indices: &Vec<u32>) -> Self {
        let vertex_buffer = Buffer::new_vertex::<ColoredVertex>(vertices.len(), device, device_memory_properties);
        vertex_buffer.fill(&vertices);
        let index_buffer = Buffer::new_index(indices.len(), device, device_memory_properties);
        index_buffer.fill(&indices);

        RawObject {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

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

    fn indices_from_block_list(block_list: &Vec<ICoords>, size: &Size3D, blocks: &Vec<BlockType>) -> Vec<u32> {
        let mut indices = vec![];

        for (i, c) in block_list.into_iter().enumerate() {
            for face in Face::all() {
                // coordinates of the neighbouring block
                let c_p = *c + face.numeric();
                if size.contains(c_p) 
                    && blocks[size.c1d(c_p) as usize] != BlockType::NoBlock {
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

    fn vertices_from_block_list(block_list: &Vec<ICoords>, position: &Vec3) -> Vec<ColoredVertex> {
        let mut vertices = vec![ColoredVertex::default(); block_list.len() * 8];
        let mut rng = thread_rng();

        for (i, c) in block_list.into_iter().enumerate() {
            let color: [f32; 4] = [rng.gen(), rng.gen(), rng.gen(), 0.8];
            // let color: [f32; 4] = [1., 1., 0., 0.8];

            for (j, [dx, dy, dz]) in BL_VERTICES.iter().enumerate() {
                let vertex = ColoredVertex {
                    pos: [
                        (c.x as u64 + dx) as f32 + position.x,
                        (c.y as u64 + dy) as f32 + position.y,
                        (c.z as u64 + dz) as f32 + position.z,
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
    fn enlist_blocks(blocks: &Vec<BlockType>, size: &Size3D) -> Vec<ICoords> {
        size.into_iter().filter(|c| blocks[size.c1d(*c) as usize] != BlockType::NoBlock).collect()
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
