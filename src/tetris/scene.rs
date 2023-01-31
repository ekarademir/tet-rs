use std::{borrow::Cow, cmp};

use wgpu::util::DeviceExt;

use super::colours;
use super::vertex::{ScreenCoord, ToVertices};

const SCREEN_WIDTH: u32 = 30; // Blocks
const SCREEN_HEIGHT: u32 = 30; // Blocks
const GAME_AREA_WIDTH: u32 = 12; // Blocks
const GAME_AREA_HEIGHT: u32 = 28; // Blocks
const LEFT_MARGIN: u32 = 1; // Blocks
const TOP_MARGIN: u32 = 1; // Blocks
const BOTTOM_MARGIN: u32 = 1; // Blocks

pub type Frame = winit::dpi::PhysicalSize<u32>;

pub struct Scene {
    pub screen_size: Frame,
    pub scene_size: Frame,
    pub block_size: u32,
    pub line_weight: u32,
    game_area_pipeline: wgpu::RenderPipeline,
}

impl<'a> Scene {
    pub fn new(base: &'a super::Base) -> Self {
        let screen_size = base.window_size.clone();

        let block_size: u32 = Scene::calculate_block_size(&screen_size);

        Scene {
            game_area_pipeline: Scene::build_game_area_pipeline(&base),
            screen_size,
            scene_size: Frame::new(SCREEN_HEIGHT * block_size, SCREEN_WIDTH * block_size),
            block_size,
            line_weight: 3,
        }
    }

    pub fn resize(&mut self, new_size: &Frame) {
        self.block_size = Scene::calculate_block_size(new_size);
        self.scene_size = Frame::new(
            SCREEN_HEIGHT * self.block_size,
            SCREEN_WIDTH * self.block_size,
        );
        self.screen_size = new_size.clone();
    }

    fn calculate_block_size(screen_size: &Frame) -> u32 {
        let block_size: u32 = cmp::min(
            screen_size.height / SCREEN_HEIGHT,
            screen_size.width / SCREEN_WIDTH,
        );

        if block_size * SCREEN_WIDTH > screen_size.width
            || block_size * SCREEN_HEIGHT > screen_size.height
        {
            if block_size > 5 {
                block_size - 5
            } else {
                0
            }
        } else {
            block_size
        }
    }

    pub fn render_game_area(&self, tetris: &super::Tetris, view: &wgpu::TextureView) {
        let (
            outer_game_area_vertex_buffer,
            outer_game_area_index_buffer,
            outer_game_area_index_buffer_len,
            inner_game_area_vertex_buffer,
            inner_game_area_index_buffer,
            inner_game_area_index_buffer_len,
        ) = {
            let outer_game_area = self.outer_game_area();

            let outer_vertices = outer_game_area.0.to_vertices(
                &self.scene_size,
                &self.screen_size,
                colours::DARK_GREEN,
            );
            let outer_indices = outer_game_area.1;

            let inner_game_area = self.inner_game_area();

            let inner_vertices =
                inner_game_area
                    .0
                    .to_vertices(&self.scene_size, &self.screen_size, colours::BLACK);
            let inner_indices = inner_game_area.1;

            (
                tetris
                    .base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&outer_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                tetris
                    .base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&outer_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                outer_indices.len() as u32,
                tetris
                    .base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&inner_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                tetris
                    .base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&inner_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                inner_indices.len() as u32,
            )
        };

        let mut encoder = tetris
            .base
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Render pass
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.game_area_pipeline);
            rpass.set_index_buffer(
                outer_game_area_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, outer_game_area_vertex_buffer.slice(..));
            rpass.draw_indexed(0..outer_game_area_index_buffer_len, 0, 0..1);
            rpass.set_index_buffer(
                inner_game_area_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, inner_game_area_vertex_buffer.slice(..));
            rpass.draw_indexed(0..inner_game_area_index_buffer_len, 0, 0..1);
        }
        tetris.base.queue.submit(Some(encoder.finish()));
    }

    fn outer_game_area(&self) -> (Vec<ScreenCoord>, Vec<u16>) {
        let (left, top, right, bottom) = {
            (
                self.block_size * LEFT_MARGIN,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH),
                self.block_size * BOTTOM_MARGIN,
            )
        };
        let (outer_left, outer_top, outer_right, outer_bottom) = {
            (
                left - self.line_weight,
                top + self.line_weight,
                right + self.line_weight,
                bottom - self.line_weight,
            )
        };
        (
            vec![
                [outer_left, outer_bottom].into(),
                [outer_right, outer_bottom].into(),
                [outer_right, outer_top].into(),
                [outer_left, outer_top].into(),
            ],
            vec![0, 1, 2, 2, 3, 0],
        )
    }

    fn inner_game_area(&self) -> (Vec<ScreenCoord>, Vec<u16>) {
        let (left, top, right, bottom) = {
            (
                self.block_size * LEFT_MARGIN,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH),
                self.block_size * BOTTOM_MARGIN,
            )
        };
        (
            vec![
                [left, bottom].into(),
                [right, bottom].into(),
                [right, top].into(),
                [left, top].into(),
            ],
            vec![0, 1, 2, 2, 3, 0],
        )
    }

    fn build_game_area_pipeline(base: &'a super::Base) -> wgpu::RenderPipeline {
        let shader = base
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("game_area.wgsl"))),
            });

        let pipeline_layout = base
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let swapchain_format = base.surface.get_supported_formats(&base.adapter)[0];

        let vertex_size = std::mem::size_of::<super::Vertex>();
        let vertex_buffers_descriptor = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Coords
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                // Colour
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        base.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers_descriptor,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
    }
}
