mod base;
mod game_state;
mod scene;
mod vertex;

use anyhow::Context;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use base::Base;
use game_state::GameState;
use vertex::Vertex;

pub struct Tetris {
    base: Base,
    game_state: GameState,
    scene: scene::Scene,
}

impl Tetris {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Tetris> {
        let base = Base::new(window)
            .await
            .context("Couldn't initialize base")?;
        let game_state = GameState::default();
        let scene = scene::Scene::new(&base);

        Ok(Tetris {
            base,
            game_state,
            scene,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.base.surface_config.width = size.width;
        self.base.surface_config.height = size.height;
        self.scene.resize(&size);
        self.base
            .surface
            .configure(&self.base.device, &self.base.surface_config);
    }

    pub fn render_game_scene(&self, view: &wgpu::TextureView) {
        let (game_area_vertex_buffer, game_area_index_buffer, game_area_index_buffer_len) = {
            let game_area = self.scene.game_area(&self.game_state);

            let vertices: Vec<_> = game_area
                .0
                .into_iter()
                .map(|x| x.to_vertex(&self.base.window_size, self.scene.left_margin))
                .collect();

            let indices = game_area.1;

            (
                self.base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.base
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                indices.len() as u32,
            )
        };

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
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.scene.game_area_pipeline);
            rpass.set_index_buffer(game_area_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, game_area_vertex_buffer.slice(..));
            rpass.draw_indexed(0..game_area_index_buffer_len, 0, 0..1);
        }
        self.base.queue.submit(Some(encoder.finish()));
    }
}

pub async fn run(
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    mut tetris: Tetris,
) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                tetris.resize(size);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = tetris
                    .base
                    .surface
                    .get_current_texture()
                    .expect("Caouldn't get next swapchain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                tetris.render_game_scene(&view);
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
