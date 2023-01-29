use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
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

pub struct ScreenCoords {
    x: u32,
    y: u32,
}

impl From<[u32; 2]> for ScreenCoords {
    fn from(value: [u32; 2]) -> Self {
        ScreenCoords {
            x: value[0],
            y: value[1],
        }
    }
}

impl ScreenCoords {
    pub fn to_vertex(
        &self,
        window_size: &winit::dpi::PhysicalSize<u32>,
        left_margin: u32,
    ) -> Vertex {
        let (x_ratio, y_ratio) = {
            let x = (left_margin + self.x) as f32 / window_size.width as f32;
            let y = self.y as f32 / window_size.height as f32;
            (2.0 * x - 1.0, 2.0 * y - 1.0)
        };

        Vertex {
            _pos: [x_ratio, y_ratio, 0.0, 1.0],
        }
    }
}
