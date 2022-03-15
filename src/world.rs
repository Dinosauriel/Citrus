use noise::{NoiseFn, Perlin};


#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct World {
    pub size: usize,
    pub vertices: Vec<Vertex>,
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
    }
}


impl Default for World {
    fn default() -> World {
        let mut w = World {
            size: 512,
            vertices: Vec::with_capacity(512 * 512),
            noise: Perlin::new(),
        };

        w.vertices.resize(w.size * w.size, Vertex{
            pos: [0.0, 0.0, 0.0, 0.0],
            color: [0.0, 1.0, 0.0, 1.0]
        });

        w.populate();

        return w;
    }
}
