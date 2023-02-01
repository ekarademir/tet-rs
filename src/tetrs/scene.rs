use std::{borrow::Cow, cmp};

use super::colours;
use super::drawable::Geometry;
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
    pub window_size: Frame,
    pub scene_size: Frame,
    pub block_size: u32,
    pub line_weight: u32,
    game_area_pipeline: wgpu::RenderPipeline,
}

impl<'a> Scene {
    pub fn new(base: &'a super::Base) -> Self {
        let window_size = base.window_size.clone();

        let block_size: u32 = Scene::calculate_block_size(&window_size);

        Scene {
            game_area_pipeline: Scene::build_game_area_pipeline(&base),
            window_size,
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
        self.window_size = new_size.clone();
    }

    fn calculate_block_size(window_size: &Frame) -> u32 {
        let block_size: u32 = cmp::min(
            window_size.height / SCREEN_HEIGHT,
            window_size.width / SCREEN_WIDTH,
        );

        if block_size * SCREEN_WIDTH > window_size.width
            || block_size * SCREEN_HEIGHT > window_size.height
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

    pub fn render_game(&self, tetrs: &super::Tetrs, view: &wgpu::TextureView) {
        let outer_rect = self
            .rectangle(
                self.block_size * LEFT_MARGIN - self.line_weight,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT) + self.line_weight,
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH) + self.line_weight,
                self.block_size * BOTTOM_MARGIN - self.line_weight,
                colours::DARK_GREEN,
            )
            .to_drawable(tetrs);
        let inner_rect = self
            .rectangle(
                self.block_size * LEFT_MARGIN,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH),
                self.block_size * BOTTOM_MARGIN,
                colours::BLACK,
            )
            .to_drawable(tetrs);

        let mut encoder = tetrs
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

            rpass.set_index_buffer(outer_rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, outer_rect.vertex_buffer.slice(..));
            rpass.draw_indexed(0..outer_rect.index_buffer_len, 0, 0..1);

            rpass.set_index_buffer(inner_rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, inner_rect.vertex_buffer.slice(..));
            rpass.draw_indexed(0..inner_rect.index_buffer_len, 0, 0..1);
        }
        tetrs.base.queue.submit(Some(encoder.finish()));
    }

    pub fn render_blocks(&self, tetrs: &super::Tetrs, view: &wgpu::TextureView) {
        let blx = self.blocks(tetrs).to_drawable(tetrs);

        let mut encoder = tetrs
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
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.game_area_pipeline);

            rpass.set_index_buffer(blx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, blx.vertex_buffer.slice(..));
            rpass.draw_indexed(0..blx.index_buffer_len, 0, 0..1);
        }
        tetrs.base.queue.submit(Some(encoder.finish()));
    }

    fn rectangle(
        &self,
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
        colour: colours::Colour,
    ) -> Geometry {
        let coords: Vec<ScreenCoord> = vec![
            [left, bottom].into(),
            [right, bottom].into(),
            [right, top].into(),
            [left, top].into(),
        ];

        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

        let vertices = coords.to_vertices(&self.scene_size, &self.window_size, colour);

        Geometry { indices, vertices }
    }

    fn blocks(&self, tetrs: &super::Tetrs) -> Geometry {
        let bs = self.block_size;
        let m: u32 = 1;

        let (ga_left, ga_top) = {
            (
                self.block_size * LEFT_MARGIN,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
            )
        };

        let mut blx = Geometry::default();

        let mut offsy = ga_top - bs;
        for row in tetrs.game_state.blocks {
            let mut offsx = ga_left;
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                if col == super::game_state::BlockState::Filled {
                    let g = self.rectangle(b_left, b_top, b_right, b_bottom, colours::ORANGE);
                    blx += g;
                }

                offsx += bs;
            }
            offsy -= bs;
        }

        blx
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
