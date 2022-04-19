use noise::{NoiseFn, Perlin};
use crate::graphics::object::Vertex;
use crate::graphics::object::TriangleGraphicsObject;

use glam::Vec3;

const BLOCK_TRIANGLE_INDICES: [usize; 36] = [
    1, 0, 2,
    1, 2, 3,
    4, 5, 6,
    5, 7, 6,
    0, 1, 4,
    1, 5, 4,
    2, 6, 3,
    3, 6, 7,
    0, 4, 2,
    2, 4, 6,
    1, 3, 5,
    3, 7, 5,
];

const BLOCK_VERTICES: [[usize; 3]; 8] = [
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 1],

    [1, 0, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 1, 1]
];

#[derive(Clone)]
pub enum BlockType {
    NoBlock,
    Grass
}

#[derive(Clone, Copy)]
struct Size3D {
    x: usize,
    y: usize,
    z: usize
}

impl Size3D {
    pub fn volume(&self) -> usize {
        return self.x * self.y * self.z;
    }

    pub fn num_vertices(&self) -> usize {
        return (self.x + 1) * (self.y + 1) * (self.z + 1);
    }

    pub fn coordinates_1_d(&self, x: usize, y: usize, z: usize) -> usize {
        return self.x * self.z * y + self.z * x + z;
    }

    pub fn vertex_coordinates_1_d(&self, x: usize, y: usize, z: usize) -> usize {
        return (self.x + 1) * (self.z + 1) * y + (self.z + 1) * x + z;
    }
}

pub struct Object {
    position: Vec3,
    // direction_x: Vec3,
    // direction_y: Vec3,
    size: Size3D,
    blocks: Vec<BlockType>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Object {
    pub fn create() -> Self {
        let size = Size3D {x: 2, y: 2, z: 2};

        let mut obj = Object {
            position: Vec3::new(10., 10., 0.),
            // direction_x: Vec3::new(1., 0., 0.),
            // direction_y: Vec3::new(0., 1., 0.),
            size: size,
            blocks: Vec::with_capacity(size.volume()),
            vertices: Vec::with_capacity(size.num_vertices()),
            indices: Vec::with_capacity(36 * size.volume()),
        };

        obj.blocks.resize(obj.blocks.capacity(), BlockType::NoBlock);
        obj.indices.resize(obj.indices.capacity(), 0);
        obj.vertices.resize_with(obj.vertices.capacity(), Default::default);

        obj.update_vertices();

        for x in 0 .. size.x {
            for y in 0 .. size.y {
                for z in 0 .. size.z {
                    let coordinates = size.coordinates_1_d(x, y, z);
                    obj.blocks[coordinates] = BlockType::Grass;

                    for i in 0 .. 36 {
                        let [d_x, d_y, d_z] = BLOCK_VERTICES[BLOCK_TRIANGLE_INDICES[i]];
                        obj.indices[coordinates * 36 + i] = size.vertex_coordinates_1_d(x + d_x, y + d_y, z + d_z) as u32;
                    }
                }
            }
        }

        return obj;
    }

    fn update_vertices(&mut self) {
        for x in 0 .. self.size.x + 1 {
            for y in 0 .. self.size.y + 1 {
                for z in 0 .. self.size.z + 1 {
                    let vertex = Vertex {
                        pos: [x as f32 + self.position.x, y as f32 + self.position.x, z as f32 + self.position.x, 1.0],
                        color: [1.0, 1.0, 0.0, 1.0]
                    };
                    self.vertices[self.size.vertex_coordinates_1_d(x, y, z)] = vertex;
                }
            }
        }
    }
}

pub struct World {
    pub size: usize,
    pub height: usize,
    pub objects: Vec<Object>,
    blocks: Vec<BlockType>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    noise: Perlin,
}

impl World {
    pub fn new(size: usize, height: usize) -> Self {

        let mut w = World {
            size: size,
            height: height,
            objects: Vec::new(),
            blocks: Vec::with_capacity(size * size * height),
            vertices: Vec::with_capacity((size + 1) * (size + 1) * (height + 1)),
            indices: Vec::with_capacity(6 * 6 * size * size),
            noise: Perlin::new(),
        };

        w.blocks.resize(w.blocks.capacity(), BlockType::NoBlock);
        w.indices.resize(w.indices.capacity(), 0);
        w.vertices.resize_with(w.vertices.capacity(), Default::default);

        w.populate();

        return w;
    }

    fn populate(&mut self) {
        for x in 0 .. self.size {
            for z in 0 .. self.size {
                let y = (5. * self.noise.get([(x as f64) / 10., (z as f64) / 10.])).floor() as usize;
                assert!(y < self.height);
                let coords = self.block_coordinates(&x, &y, &z);
                self.blocks[coords] = BlockType::Grass;

                for i in 0 .. BLOCK_TRIANGLE_INDICES.len() {
                    let [v_x, v_y, v_z] = BLOCK_VERTICES[BLOCK_TRIANGLE_INDICES[i]];
                    self.indices[(x * self.size + z) * BLOCK_TRIANGLE_INDICES.len() + i] = ((y + v_y) * (self.size + 1) * (self.size + 1) + (x + v_x) * (self.size + 1) + (z + v_z)) as u32;
                }
            }
        }

        for y in 0 .. self.height + 1 {
            for x in 0 .. self.size + 1 {
                for z in 0 .. self.size + 1 {
                    self.vertices[y * (self.size + 1) * (self.size + 1) + x * (self.size + 1) + z] = Vertex {
                        pos: [x as f32, y as f32, z as f32, 1.0],
                        color: [0.0, 1.0, 0.0, 1.0]
                    }
                }
            }
        }
    }

    pub fn block_coordinates(&self, x: &usize, y: &usize, z: &usize) -> usize {
        return y * self.size * self.size + x * self.size + z;
    }

    pub fn block_at(&self, x: &usize, y: &usize, z: &usize) -> &BlockType {
        if *x >= self.size || *y >= self.height || *z >= self.size {
            return &BlockType::NoBlock;
        }
        return &self.blocks[self.block_coordinates(x, y, z)];
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