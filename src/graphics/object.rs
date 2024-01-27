use super::{buffer::Buffer, vertex::*};

pub trait GraphicsObject<T: Vertex> {
    fn vertices(&self) -> &Vec<T>;
    fn indices(&self) -> &Vec<u32>;

    unsafe fn index_buffer<'a>(&self, device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) -> Buffer<'a> {
        let buffer = Buffer::create(
            &device,
            &device_memory_properties,
            (self.indices().len() * std::mem::size_of::<u32>()) as u64,
            ash::vk::BufferUsageFlags::INDEX_BUFFER,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT);
        buffer.fill( self.indices());
        return buffer;
    }
    
    unsafe fn vertex_buffer<'a>(&self, device: &'a ash::Device, device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties) -> Buffer<'a> {
        let buffer = Buffer::create(
            &device,
            &device_memory_properties,
            (self.vertices().len() * std::mem::size_of::<T>()) as u64, 
            ash::vk::BufferUsageFlags::VERTEX_BUFFER,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT);
        buffer.fill(self.vertices());
        return buffer;
    }
}

pub struct Triangle<T: Vertex> {
    vertices: Vec<T>,
    indices: Vec<u32>
}

impl<T: Vertex> Triangle<T> {
    pub fn create(point_a: &T, point_b: &T, point_c: &T) -> Triangle<T> {
        Triangle {
            vertices:  vec![*point_a, *point_b, *point_c],
            indices: vec![0, 1, 2]
        }
    }
}

impl<T: Vertex> GraphicsObject<T> for Triangle<T> {
    fn vertices(&self) -> &Vec<T> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}
