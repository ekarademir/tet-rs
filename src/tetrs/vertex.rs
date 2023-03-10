use bytemuck::{Pod, Zeroable};

use super::colours;
use super::scene::Frame;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _colour: [f32; 4],
}

impl From<[f32; 2]> for Vertex {
    fn from(value: [f32; 2]) -> Self {
        Vertex {
            _pos: [value[0], value[1], 0.0, 1.0],
            _colour: colours::DARK_GREEN.into(),
        }
    }
}

impl From<[f32; 3]> for Vertex {
    fn from(value: [f32; 3]) -> Self {
        Vertex {
            _pos: [value[0], value[1], value[2], 1.0],
            _colour: colours::DARK_GREEN.into(),
        }
    }
}

impl From<[f32; 4]> for Vertex {
    fn from(value: [f32; 4]) -> Self {
        Vertex {
            _pos: [value[0], value[1], value[2], value[3]],
            _colour: colours::DARK_GREEN.into(),
        }
    }
}

impl From<[f32; 8]> for Vertex {
    fn from(value: [f32; 8]) -> Self {
        Vertex {
            _pos: [value[0], value[1], value[2], value[3]],
            _colour: [value[4], value[5], value[6], value[7]],
        }
    }
}

impl From<([f32; 2], super::colours::Colour)> for Vertex {
    fn from(value: ([f32; 2], super::colours::Colour)) -> Self {
        Vertex {
            _pos: [value.0[0], value.0[1], 0.0, 1.0],
            _colour: value.1.into(),
        }
    }
}

#[derive(Debug)]
pub struct ScreenCoord {
    x: u32,
    y: u32,
}

impl From<[u32; 2]> for ScreenCoord {
    fn from(value: [u32; 2]) -> Self {
        ScreenCoord {
            x: value[0],
            y: value[1],
        }
    }
}

impl ScreenCoord {
    pub fn to_vertex(
        &self,
        scene_size: &Frame,
        window_size: &Frame,
        colour: colours::Colour,
    ) -> Vertex {
        let (left_margin, bottom_margin) = {
            (
                (window_size.width - scene_size.width) / 2,
                (window_size.height - scene_size.height) / 2,
            )
        };
        let (x_ratio, y_ratio) = {
            let x = (left_margin + self.x) as f32 / window_size.width as f32;
            let y = (self.y + bottom_margin) as f32 / window_size.height as f32;
            (2.0 * x - 1.0, 2.0 * y - 1.0)
        };

        Vertex {
            _pos: [x_ratio, y_ratio, 0.0, 1.0],
            _colour: colour.into(),
        }
    }
}

pub trait ToVertices {
    fn to_vertices(
        &self,
        scene_size: &Frame,
        window_size: &Frame,
        colour: colours::Colour,
    ) -> Vec<Vertex>;
}

impl ToVertices for Vec<ScreenCoord> {
    fn to_vertices(
        &self,
        scene_size: &Frame,
        window_size: &Frame,
        colour: colours::Colour,
    ) -> Vec<Vertex> {
        self.into_iter()
            .map(|x| x.to_vertex(&scene_size, &window_size, colour))
            .collect()
    }
}
