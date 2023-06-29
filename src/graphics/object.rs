use super::vertex::*;

pub trait GraphicsObject<T: Vertex> {
    fn vertices(&self) -> &Vec<T>;
    fn indices(&self) -> &Vec<u32>;
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
