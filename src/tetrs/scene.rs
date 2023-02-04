use std::{borrow::Cow, cmp};

use anyhow::Context;
use wgpu_text::section::{
    BuiltInLineBreaker, Color, HorizontalAlign, Layout, Section, Text, VerticalAlign,
};

use super::base::Base;
use super::colours;
use super::drawable::{Drawable, Geometry};
use super::vertex::Vertex;
use super::vertex::{ScreenCoord, ToVertices};
use super::writer::Writer;
use super::{game_state, game_state::BlockState, game_state::Tetromino};

pub const SCREEN_WIDTH: u32 = 30; // Blocks
pub const SCREEN_HEIGHT: u32 = 30; // Blocks
pub const GAME_AREA_WIDTH: u32 = 12; // Blocks
pub const GAME_AREA_HEIGHT: u32 = 28; // Blocks
pub const LEFT_MARGIN: u32 = 1; // Blocks
pub const TOP_MARGIN: u32 = 1; // Blocks
pub const SPACE: u32 = 1; // Blocks
pub const BOTTOM_MARGIN: u32 = 1; // Blocks

pub type Frame = winit::dpi::PhysicalSize<u32>;

pub struct Scene {
    base: Base,
    block_size: u32,
    line_weight: u32,
    scene_size: Frame,
    window_size: Frame,
    pipeline: wgpu::RenderPipeline,
    writer: Writer,
}

impl<'a> Scene {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let base = Base::new(window)
            .await
            .context("Couldn't initialize base")?;
        let window_size = base.window_size.clone();

        let block_size: u32 = Scene::calculate_block_size(&window_size);

        let writer = Writer::new(&base).context("Couldn't create the text writer")?;

        Ok(Scene {
            pipeline: Scene::build_pipeline(&base),
            window_size,
            scene_size: Frame::new(SCREEN_HEIGHT * block_size, SCREEN_WIDTH * block_size),
            block_size,
            line_weight: 12,
            writer,
            base,
        })
    }

    pub fn get_next_frame(&self) -> wgpu::SurfaceTexture {
        self.base
            .surface
            .get_current_texture()
            .expect("Couldn't get next swapchain texture")
    }

    pub fn resize(&mut self, new_size: &Frame) {
        self.base.surface_config.width = new_size.width;
        self.base.surface_config.height = new_size.height;
        self.base
            .surface
            .configure(&self.base.device, &self.base.surface_config);
        self.block_size = Scene::calculate_block_size(new_size);
        self.scene_size = Frame::new(
            SCREEN_HEIGHT * self.block_size,
            SCREEN_WIDTH * self.block_size,
        );
        self.window_size = new_size.clone();
        self.writer.brush.resize_view(
            new_size.width as f32,
            new_size.height as f32,
            &self.base.queue,
        );
    }

    pub fn render_next(&mut self, view: &wgpu::TextureView, game_state: &super::GameState) {
        self.write(&view, "next", SPACE * 1);
        let blx = self
            .next_tetromino_geom(&game_state::Ell)
            .to_drawable(&self.base);
        self.draw_blocks(view, blx);
    }

    pub fn render_score(&mut self, view: &wgpu::TextureView, game_state: &super::GameState) {
        let text = format!("score   {}", game_state.score);
        self.write(&view, text.as_str(), SPACE * 12);
    }

    pub fn render_level(&mut self, view: &wgpu::TextureView, game_state: &super::GameState) {
        let text = format!("level   {}", game_state.level);
        self.write(&view, text.as_str(), SPACE * 14);
    }

    pub fn render_game(&self, view: &wgpu::TextureView) {
        let outer_rect = self
            .rectangle(
                self.block_size * LEFT_MARGIN - self.line_weight,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT) + self.line_weight,
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH) + self.line_weight,
                self.block_size * BOTTOM_MARGIN - self.line_weight,
                colours::DARK_GREEN,
            )
            .to_drawable(&self.base);
        let inner_rect = self
            .rectangle(
                self.block_size * LEFT_MARGIN,
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH),
                self.block_size * BOTTOM_MARGIN,
                colours::BLACK,
            )
            .to_drawable(&self.base);

        let mut encoder = self
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);

            rpass.set_index_buffer(outer_rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, outer_rect.vertex_buffer.slice(..));
            rpass.draw_indexed(0..outer_rect.index_buffer_len, 0, 0..1);

            rpass.set_index_buffer(inner_rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, inner_rect.vertex_buffer.slice(..));
            rpass.draw_indexed(0..inner_rect.index_buffer_len, 0, 0..1);
        }
        self.base.queue.submit(Some(encoder.finish()));
    }

    pub fn render_blocks(&self, view: &wgpu::TextureView, game_state: &super::GameState) {
        let blx = self.blocks(game_state).to_drawable(&self.base);
        self.draw_blocks(view, blx);
    }

    fn draw_blocks(&self, view: &wgpu::TextureView, blx: Drawable) {
        let mut encoder = self
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

            rpass.set_pipeline(&self.pipeline);

            rpass.set_index_buffer(blx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, blx.vertex_buffer.slice(..));
            rpass.draw_indexed(0..blx.index_buffer_len, 0, 0..1);
        }
        self.base.queue.submit(Some(encoder.finish()));
    }

    fn write(&mut self, view: &wgpu::TextureView, text: &str, y_blocks: u32) {
        let (left_margin, top_margin) = {
            (
                (self.window_size.width - self.scene_size.width) / 2,
                (self.window_size.height - self.scene_size.height) / 2,
            )
        };

        let font_size = self.block_size as f32;
        let colour: Color = super::colours::LIGHT_BLUE.into();
        let pos_x = (LEFT_MARGIN + GAME_AREA_WIDTH + SPACE) * self.block_size + left_margin;
        let pos_y = (TOP_MARGIN + y_blocks + SPACE) * self.block_size + top_margin;
        let section = Section::default()
            .add_text(Text::new(text).with_scale(font_size).with_color(colour))
            .with_bounds((
                (GAME_AREA_WIDTH * self.block_size) as f32,
                (GAME_AREA_HEIGHT * self.block_size) as f32,
            ))
            .with_layout(
                Layout::default_single_line()
                    .h_align(HorizontalAlign::Left)
                    .v_align(VerticalAlign::Top)
                    .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .with_screen_position((pos_x as f32, pos_y as f32))
            .to_owned();

        let mut encoder = self
            .base
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Render pass
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }
        self.writer.brush.queue(&section);
        let cmd_buffer = self
            .writer
            .brush
            .draw(&self.base.device, &view, &self.base.queue);
        self.base.queue.submit([encoder.finish(), cmd_buffer]);
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

    fn next_tetromino_geom<const R: usize, const C: usize>(
        &self,
        tetromino: &Tetromino<R, C>,
    ) -> Geometry {
        let (ga_left, ga_top) = {
            (
                self.block_size * (LEFT_MARGIN + GAME_AREA_WIDTH + 3 * SPACE / 2),
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT - 3 * SPACE),
            )
        };

        let mut blx = Geometry::default();

        let bs = self.block_size;
        let m: u32 = 1;

        let mut offsy = ga_top - bs;
        for row in tetromino.shape {
            let mut offsx = ga_left;
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                if col != BlockState::Emp {
                    let g = self.rectangle(b_left, b_top, b_right, b_bottom, tetromino.colour);
                    blx += g;
                }

                offsx += bs;
            }
            offsy -= bs;
        }

        blx
    }

    fn blocks(&self, game_state: &super::GameState) -> Geometry {
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
        for row in game_state.blocks {
            let mut offsx = ga_left;
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                if col == BlockState::Filled {
                    let g = self.rectangle(b_left, b_top, b_right, b_bottom, colours::ORANGE);
                    blx += g;
                }

                offsx += bs;
            }
            offsy -= bs;
        }

        blx
    }

    fn build_pipeline(base: &'a Base) -> wgpu::RenderPipeline {
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

        let vertex_size = std::mem::size_of::<Vertex>();
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
