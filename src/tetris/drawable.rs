use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Drawable {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_len: u32,
}

#[derive(Debug)]
pub struct Geometry {
    pub colour: super::colours::Colour,
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
