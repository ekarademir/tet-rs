use bytemuck::{Pod, Zeroable};

use super::scene::Frame;

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

#[derive(Debug)]
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
    pub fn to_vertex(&self, scene_size: &Frame, window_size: &Frame) -> Vertex {
        let (left_margin, bottom_margin) = {
            (
                window_size.width - scene_size.width,
                window_size.height - scene_size.height,
            )
        };
        let (x_ratio, y_ratio) = {
            let x = (left_margin + self.x) as f32 / window_size.width as f32;
            let y = (self.y + bottom_margin) as f32 / window_size.height as f32;
            (2.0 * x - 1.0, 2.0 * y - 1.0)
        };

        Vertex {
            _pos: [x_ratio, y_ratio, 0.0, 1.0],
        }
    }
}

pub trait Scaleable<T> {
    fn to_vertices(xs: T, scene_size: &Frame, window_size: &Frame) -> Vec<Vertex>;
}

impl Scaleable<Vec<ScreenCoords>> for Vec<ScreenCoords> {
    fn to_vertices(xs: Vec<ScreenCoords>, scene_size: &Frame, window_size: &Frame) -> Vec<Vertex> {
        xs.into_iter()
            .map(|x| x.to_vertex(&scene_size, &window_size))
            .collect()
    }
}
