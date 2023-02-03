use wgpu::util::DeviceExt;

use super::base::Base;
use super::vertex::Vertex;

#[derive(Debug)]
pub struct Drawable {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_len: u32,
}

#[derive(Debug)]
pub struct Geometry {
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            indices: Vec::new(),
            vertices: Vec::new(),
        }
    }
}

impl Geometry {
    pub fn to_drawable(&self, base: &Base) -> Drawable {
        let vertex_buffer = base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let index_buffer_len = self.indices.len() as u32;

        Drawable {
            vertex_buffer,
            index_buffer,
            index_buffer_len,
        }
    }
}

impl std::ops::Add for Geometry {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let offset = self.vertices.len() as u16;

        let vertices = [self.vertices, rhs.vertices].concat();

        let indices = [
            self.indices,
            rhs.indices.into_iter().map(|x| x + offset).collect(),
        ]
        .concat();

        Geometry { vertices, indices }
    }
}
impl std::ops::AddAssign for Geometry {
    fn add_assign(&mut self, rhs: Self) {
        let offset = self.vertices.len() as u16;

        self.vertices.extend_from_slice(&rhs.vertices);

        self.indices
            .extend(rhs.indices.into_iter().map(|x| x + offset));
    }
}
