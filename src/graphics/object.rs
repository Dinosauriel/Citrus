
#[derive(Clone, Debug, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct Vertex2D {
    pub pos: [f32; 3],
    pub color: [f32; 4]
}

pub trait TriangleGraphicsObject {
    fn vertices(&self) -> &Vec<Vertex>;
    fn indices(&self) -> &Vec<u32>;
}