use crate::graphics::buffer::Buffer;
use crate::graphics::vertex::ColoredVertex;

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
