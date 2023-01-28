use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Vertex {
    _pos: [f32; 4],
}

impl From<[f32; 2]> for Vertex {
    fn from(value: [f32; 2]) -> Self {
        Vertex {
            _pos: [value[0], value[1], 0.0, 1.0],
        }
    }
}

impl From<[f32; 3]> for Vertex {
    fn from(value: [f32; 3]) -> Self {
        Vertex {
            _pos: [value[0], value[1], value[2], 1.0],
        }
    }
}

impl From<[f32; 4]> for Vertex {
    fn from(value: [f32; 4]) -> Self {
        Vertex {
            _pos: [value[0], value[1], value[2], value[3]],
        }
    }
}
