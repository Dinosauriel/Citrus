use crate::graphics::object::Vertex;
use crate::graphics::object::TriangleGraphicsObject;

pub struct Triangle {
    vertices: Vec<Vertex>,
    indices: Vec<u32>
}

impl Triangle {
    pub fn new(point_a: &Vertex, point_b: &Vertex, point_c: &Vertex) -> Triangle {
        return Triangle {
            vertices:  vec![*point_a, *point_b, *point_c],
            indices: vec![0, 1, 2]
        };

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