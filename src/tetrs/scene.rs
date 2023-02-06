use std::{borrow::Cow, cmp};

use anyhow::Context;
use wgpu_text::section::{
    BuiltInLineBreaker, Color, HorizontalAlign, Layout, Section, Text, VerticalAlign,
};

use super::base::Base;
use super::colours;
use super::drawable::{Drawable, Geometry};
use super::game_state;
use super::tetromino::{BlockState, CurrentTetromino, Tetromino};
use super::vertex::Vertex;
use super::vertex::{ScreenCoord, ToVertices};
use super::writer::Writer;

pub const SCREEN_WIDTH: u32 = 30; // Blocks
pub const SCREEN_HEIGHT: u32 = 30; // Blocks
pub const GAME_AREA_WIDTH: u32 = game_state::NUM_COLS as u32; // Blocks
pub const GAME_AREA_HEIGHT: u32 = game_state::NUM_ROWS as u32; // Blocks
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

    pub fn render_next_tetromino(
        &mut self,
        view: &wgpu::TextureView,
        game_state: &super::GameState,
    ) {
        self.write(&view, "next", SPACE * 1, GAME_AREA_WIDTH);
        let blx = self
            .next_tetromino_geom(&game_state.next_tetromino.tetromino)
            .to_drawable(&self.base);
        self.draw_blocks(view, blx);
    }

    pub fn render_current_tetromino(
        &mut self,
        view: &wgpu::TextureView,
        game_state: &super::GameState,
    ) {
        let blx = self
            .current_tetromino_geom(&game_state.current_tetromino)
            .to_drawable(&self.base);
        self.draw_blocks(view, blx);
    }

    pub fn render_score(&mut self, view: &wgpu::TextureView, game_state: &super::GameState) {
        let text = format!("score   {}", game_state.score);
        self.write(&view, text.as_str(), SPACE * 12, GAME_AREA_WIDTH);
    }

    pub fn render_pause(&mut self, view: &wgpu::TextureView, _game_state: &super::GameState) {
        self.write(&view, "PAUSED", SPACE * 12, SPACE * 3);
    }

    pub fn render_level(&mut self, view: &wgpu::TextureView, game_state: &super::GameState) {
        let text = format!("level   {}", game_state.level);
        self.write(&view, text.as_str(), SPACE * 14, GAME_AREA_WIDTH);
    }

    pub fn render_debug(&mut self, view: &wgpu::TextureView, to_dbg: &String) {
        self.write(&view, &to_dbg.as_str(), SPACE * 20, GAME_AREA_WIDTH);
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

    fn write(&mut self, view: &wgpu::TextureView, text: &str, y_blocks: u32, x_blocks: u32) {
        let (left_margin, top_margin) = {
            (
                (self.window_size.width - self.scene_size.width) / 2,
                (self.window_size.height - self.scene_size.height) / 2,
            )
        };

        let font_size = self.block_size as f32;
        let colour: Color = super::colours::LIGHT_BLUE.into();
        let pos_x = (LEFT_MARGIN + x_blocks + SPACE) * self.block_size + left_margin;
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

    fn next_tetromino_geom(&self, tetromino: &Tetromino) -> Geometry {
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
        for row in &tetromino.shape {
            let mut offsx = ga_left;
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                if *col != BlockState::Emp {
                    let g = self.rectangle(b_left, b_top, b_right, b_bottom, tetromino.colour);
                    blx += g;
                }

                offsx += bs;
            }
            offsy -= bs;
        }

        blx
    }

    fn current_tetromino_geom(&self, current_tetromino: &CurrentTetromino) -> Geometry {
        // Determine the bounding box for the game area
        let (ga_left, ga_top) = {
            (
                self.block_size * (LEFT_MARGIN),
                self.block_size * (BOTTOM_MARGIN + GAME_AREA_HEIGHT),
            )
        };

        // Some bin to collect all gemotries, by default an empty one.
        let mut blx = Geometry::default();

        // Block related drawing dimentions.
        let bs = self.block_size;
        let m: u32 = 1;

        // Determining how much of the tetromino should be in view
        // `current_tetromino.y` is already rebated for the height of the tetromino we first add it back.
        // Then the resulting height should be rebated for the `UNRENDERED_HEIGHT`, since
        // tetrominos start from outside of the "rendered" box, this then gives the amount
        // of tetromino `in_view`.
        // Finally we return the index bounds of the rows of the shape that should be rendered.
        let shape_height = current_tetromino.tetromino.shape.len() as i8;
        let (in_view_row_start, in_view_row_end, in_view): (usize, usize, usize) = {
            let in_view = current_tetromino.y + shape_height;
            if in_view < shape_height {
                (
                    (shape_height - in_view) as usize,
                    shape_height as usize,
                    in_view as usize,
                )
            } else {
                (0, shape_height as usize, shape_height as usize)
            }
        };

        let dy = in_view as u32;
        let dh = (current_tetromino.y + shape_height) as u32;

        let mut offsy = ga_top - bs * (dh - dy + 1);
        for row in &current_tetromino.tetromino.shape[in_view_row_start..in_view_row_end] {
            let mut offsx = ga_left + bs * (current_tetromino.x as u32);
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                if *col != BlockState::Emp {
                    let g = self.rectangle(
                        b_left,
                        b_top,
                        b_right,
                        b_bottom,
                        current_tetromino.tetromino.colour,
                    );
                    blx += g;
                }

                offsx += bs;
            }
            offsy = if offsy > 0 { offsy - bs } else { offsy };
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
        for row in &game_state.blocks[..] {
            let mut offsx = ga_left;
            for col in row {
                let (b_left, b_top, b_right, b_bottom) =
                    { (offsx + m, offsy + m, offsx + bs - m, offsy + bs - m) };

                // Ignore the part of the blocks where we use for injecting new tetrominos off-screen
                if *col != BlockState::Emp {
                    let colour = match col {
                        BlockState::Arr => Tetromino::arr().colour,
                        BlockState::Ell => Tetromino::ell().colour,
                        BlockState::Ess => Tetromino::ess().colour,
                        BlockState::Eye => Tetromino::eye().colour,
                        BlockState::Ohh => Tetromino::ohh().colour,
                        BlockState::Tee => Tetromino::tee().colour,
                        BlockState::Zee => Tetromino::zee().colour,
                        _ => colours::UNRENDERED,
                    };
                    let g = self.rectangle(b_left, b_top, b_right, b_bottom, colour);
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
