use super::{buffer::Buffer, vertex::*};

// an object that provides an array of vertices and an array of indices for graphics rendering
pub trait GraphicsObject<'a, T: Vertex> {
    fn vertices(&self) -> &Vec<T>;
    fn indices(&self) -> &Vec<u32>;
    fn vertex_buffer(&self) -> &Buffer<'a>;
    fn index_buffer(&self) -> &Buffer<'a>;
}

pub struct Triangle<'a, T: Vertex> {
    vertices: Vec<T>,
    indices: Vec<u32>,
    vertex_buffer: Buffer<'a>,
    index_buffer: Buffer<'a>,
}

impl<'a, T: Vertex> Triangle<'a, T> {
    pub unsafe fn new(device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
                point_a: &T, point_b: &T, point_c: &T) -> Triangle<'a, T> {
        let vertices = vec![*point_a, *point_b, *point_c];
        let indices = vec![0, 1, 2];
        let vertex_buffer = Buffer::new_vertex::<T>(vertices.len(), device, device_memory_properties);
        vertex_buffer.fill(&vertices);
        let index_buffer = Buffer::new_index(indices.len(), device, device_memory_properties);
        index_buffer.fill(&indices);
        Triangle {
            vertices,
            indices,
            vertex_buffer,
            index_buffer
        }
    }
}

impl<'a, T: Vertex> GraphicsObject<'a, T> for Triangle<'a, T> {
    fn vertex_buffer(&self) -> &Buffer<'a> {
        &self.vertex_buffer
    }

    fn index_buffer(&self) -> &Buffer<'a> {
        &self.index_buffer
    }

    fn vertices(&self) -> &Vec<T> {
        &self.vertices
    }

    fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}
