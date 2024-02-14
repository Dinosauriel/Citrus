use std::mem;
use ash::vk;
use crate::offset_of;

// Clone, Copy, and Default are "supertraits" of Vertex
pub trait Vertex: Clone + Copy + Default {
    fn binding_description<'a>() -> [vk::VertexInputBindingDescription; 1];
    fn attribute_desctiptions<'a>() -> [vk::VertexInputAttributeDescription; 2];
}

// use the C representation because the Default rust representation may reorder fields
// a vertex with a color attribute
#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct ColoredVertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex for ColoredVertex {
    fn binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<ColoredVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_desctiptions() -> [vk::VertexInputAttributeDescription; 2] {
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
    pub const ZERO: Self = TexturedVertex {
        pos: [0.; 4],
        tex_coord: [0.; 2]
    };
}

impl Vertex for TexturedVertex {
    fn binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<TexturedVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_desctiptions() -> [vk::VertexInputAttributeDescription; 2] {
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
