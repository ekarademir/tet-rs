use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Drawable {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_len: u32,
}

#[derive(Debug)]
pub struct Geometry {
    pub indices: Vec<u16>,
    pub vertices: Vec<super::Vertex>,
}

impl Geometry {
    pub fn to_drawable(&self, tetris: &super::Tetris) -> Drawable {
        let vertex_buffer =
            tetris
                .base
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let index_buffer =
            tetris
                .base
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
        let vertices = [self.vertices, rhs.vertices].concat();

        let offset = self.indices.len() as u16;
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
        self.vertices.extend_from_slice(&rhs.vertices);

        let offset = self.indices.len() as u16;
        self.indices
            .extend(rhs.indices.into_iter().map(|x| x + offset));
    }
}
