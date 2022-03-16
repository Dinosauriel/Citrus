use noise::{NoiseFn, Perlin};
use crate::graphics::object::Vertex;
use crate::graphics::object::TriangleGraphicsObject;

pub struct World {
    pub size: usize,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    noise: Perlin,
}

impl World {
    fn populate(&mut self) {
        for i in 0 .. self.size {
            for j in 0 .. self.size {
                let y = (2. * self.noise.get([(i as f64) / 10., (j as f64) / 10.])).floor();
                // println!("{:?}", y);
                let v = Vertex {
                    pos: [i as f32, y as f32, j as f32, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                };

                self.vertices[self.size * i + j] = v;
            }
        }


        for i in 0 .. self.size - 1 {
            for j in 0 .. self.size - 1 {
                let tl = i       * self.size + j;
                let bl = (i + 1) * self.size + j;
                let tr = i       * self.size + j + 1;
                let br = (i + 1) * self.size + j + 1;

                let k = 6 * (i * (self.size - 1) + j);
                self.indices[k + 0] = tl as u32;
                self.indices[k + 1] = bl as u32;
                self.indices[k + 2] = tr as u32;
                self.indices[k + 3] = tr as u32;
                self.indices[k + 4] = bl as u32;
                self.indices[k + 5] = br as u32;
            }
        }
    }

    pub fn new(size: usize) -> Self {
        let mut w = World {
            size: size,
            vertices: Vec::with_capacity(size * size),
            indices: Vec::with_capacity(6 * (size - 1) * (size - 1)),
            noise: Perlin::new(),
        };

        w.indices.resize_with(6 * (size - 1) * (size - 1), Default::default);
        w.vertices.resize_with(w.size * w.size, Default::default);

        w.populate();

        return w;
    }
}

impl TriangleGraphicsObject for World {
    fn vertices(&self) -> &Vec<Vertex> {
        return &self.vertices;
    }

    fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }
}