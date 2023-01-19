use std::mem;
use ash::vk;
use crate::offset_of;

// use the C representation because the Default rust representation may reorder fields
#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_desctiptions() -> [vk::VertexInputAttributeDescription; 3] {
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, pos) as u32,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, color) as u32,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex, tex_coord) as u32,
        }]
    }
}

// a vertex with a color attribute
#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct ColoredVertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl ColoredVertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<ColoredVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_desctiptions() -> [vk::VertexInputAttributeDescription; 2] {
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(ColoredVertex, pos) as u32,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(ColoredVertex, color) as u32,
        }]
    }
}

// a vertex with texture coordinates attribute
#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct TexturedVertex {
    pub pos: [f32; 4],
    pub tex_coord: [f32; 2],
}

impl TexturedVertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<TexturedVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_desctiptions() -> [vk::VertexInputAttributeDescription; 2] {
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(TexturedVertex, pos) as u32,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(TexturedVertex, tex_coord) as u32,
        }]
    }
}
