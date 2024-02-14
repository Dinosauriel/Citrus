use crate::graphics::buffer::Buffer;
use crate::graphics::vertex::ColoredVertex;
use crate::world::*;
use crate::world::block::*;
use rand::prelude::*;

pub struct BlockObject<'a> {
    position: Vec3,
    size: Size3D,
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<u32>,
    vertex_buffer: Buffer<'a>,
    index_buffer: Buffer<'a>,

    pub is_ticking: bool,
}

impl<'a> BlockObject<'a> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
            size: &Size3D, position: &Vec3, blocks: &Vec<BlockType>) -> Self {

        let block_list = Self::block_list(&blocks, &size);
        let vertices = Self::vertices_from(&block_list, &position);
        let indices = Self::indices_from(&block_list, &size, &blocks);

        let vertex_buffer = Buffer::new_vertex::<ColoredVertex>(vertices.len(), device, device_memory_properties);
        vertex_buffer.fill(&vertices);
        let index_buffer = Buffer::new_index(indices.len(), device, device_memory_properties);
        index_buffer.fill(&indices);

        BlockObject {
            position: *position,
            size: *size,
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

    fn indices_from(block_list: &Vec<(usize, usize, usize)>, size: &Size3D, blocks: &Vec<BlockType>) -> Vec<u32> {
        let mut indices = vec![];

        for (i, (x, y, z)) in block_list.into_iter().enumerate() {
            let c = ICoords {x: *x as i64, y: *y as i64, z: *z as i64};
            for face in Face::all() {
                // coordinates of the neighbouring block
                let c_p = c + face.numeric();
                if size.contains(c_p) 
                    && blocks[size.coordinates_1_d(c_p.x as usize, c_p.y as usize, c_p.z as usize)] != BlockType::NoBlock {
                        // skip these indices if the neighbouring coordinates are not empty
                        continue;
                }

                for index in face.indices() {
                    indices.push((i * 8 + index) as u32)
                }
            }
        }
        indices
    }

    fn vertices_from(block_list: &Vec<(usize, usize, usize)>, position: &Vec3) -> Vec<ColoredVertex> {
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

    fn block_list(blocks: &Vec<BlockType>, size: &Size3D) -> Vec<(usize, usize, usize)> {
        let mut list = vec![(0, 0, 0); 0];
        for (x, y, z) in size.into_iter() {
            if blocks[size.coordinates_1_d(x, y, z)] != BlockType::NoBlock {
                list.push((x, y, z));
            }
        }
        return list;
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
