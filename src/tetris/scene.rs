use std::{borrow::Cow, cmp};

use super::vertex::ScreenCoords;

const SCREEN_WIDTH: u32 = 30; // Blocks
const SCREEN_HEIGHT: u32 = 30; // Blocks
const GAME_AREA_WIDTH: u32 = 12; // Blocks
const GAME_AREA_HEIGHT: u32 = 28; // Blocks
const LEFT_MARGIN: u32 = 1; // Blocks
const TOP_MARGIN: u32 = 1; // Blocks
const BOTTOM_MARGIN: u32 = 1; // Blocks

pub type AbstractSize = winit::dpi::PhysicalSize<u32>;

pub struct Scene {
    pub game_area_pipeline: wgpu::RenderPipeline,
    pub screen_size: AbstractSize,
    pub scene_size: AbstractSize,
    pub block_size: u32,
    pub left_margin: u32,
    pub line_weight: u32,
}

impl<'a> Scene {
    pub fn new(base: &'a super::Base) -> Self {
        let block_size: u32 = cmp::min(
            base.window_size.height / SCREEN_HEIGHT,
            base.window_size.width / SCREEN_WIDTH,
        );
        Scene {
            game_area_pipeline: Scene::build_game_area_pipeline(&base),
            screen_size: base.window_size.clone(),
            scene_size: AbstractSize::new(SCREEN_HEIGHT * block_size, SCREEN_WIDTH * block_size),
            block_size,
            left_margin: 0,
            line_weight: 2,
        }
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        let block_size: u32 = cmp::min(
            new_size.height / SCREEN_HEIGHT,
            new_size.width / SCREEN_WIDTH,
        );
        self.block_size = block_size;
        self.left_margin = 0;
        self.scene_size = AbstractSize::new(SCREEN_HEIGHT * block_size, SCREEN_WIDTH * block_size);
        self.screen_size = new_size.clone();
    }

    pub fn game_area(&self, game_state: &super::GameState) -> (Vec<ScreenCoords>, Vec<u16>) {
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
                // Left bar
                [outer_left, outer_bottom].into(),
                [left, outer_bottom].into(),
                [left, outer_top].into(),
                [outer_left, outer_top].into(),
                // Top bar
                [left, top].into(),
                [right, top].into(),
                [right, outer_top].into(),
                [left, outer_top].into(),
                // Right bar
                [right, outer_bottom].into(),
                [outer_right, outer_bottom].into(),
                [outer_right, outer_top].into(),
                [right, outer_top].into(),
                // Bottom bar
                [outer_left, outer_bottom].into(),
                [outer_right, outer_bottom].into(),
                [outer_right, bottom].into(),
                [outer_left, bottom].into(),
            ],
            vec![
                0, 1, 2, 2, 3, 0, // Left bar
                4, 5, 6, 6, 7, 4, // Top bar
                8, 9, 10, 10, 11, 8, // Right bar
                12, 13, 14, 14, 15, 12, // Bottom bar
            ],
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
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
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
