
#[derive(Clone, Debug, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

pub trait TriangleGraphicsObject {
    fn vertices(&self) -> &Vec<Vertex>;
    fn indices(&self) -> &Vec<u32>;
}

pub struct Triangle {
    vertices: Vec<Vertex>,
    indices: Vec<u32>
}

impl Triangle {
    pub fn create(point_a: &Vertex, point_b: &Vertex, point_c: &Vertex) -> Triangle {
        Triangle {
            vertices:  vec![*point_a, *point_b, *point_c],
            indices: vec![0, 1, 2]
        }
    }
}

impl TriangleGraphicsObject for Triangle {
    fn vertices(&self) -> &Vec<Vertex> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}