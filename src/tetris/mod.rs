mod game_state;
mod scene;
mod vertex;

use std::borrow::Cow;

use anyhow::Context;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use game_state::GameState;
use vertex::Vertex;

#[allow(dead_code)]
pub struct Base {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window_size: winit::dpi::PhysicalSize<u32>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl Base {
    async fn new(window: &winit::window::Window) -> anyhow::Result<Base> {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Couldn't obtain an adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .context("Couldn't create logical device and job queue")?;

        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(&adapter)[0],
        };

        surface.configure(&device, &config);

        Ok(Base {
            window_size,
            instance,
            surface,
            surface_config: config,
            adapter,
            device,
            queue,
        })
    }
}

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
                tetris.base.surface_config.width = size.width;
                tetris.base.surface_config.height = size.height;
                tetris.scene.resize(&size);
                tetris
                    .base
                    .surface
                    .configure(&tetris.base.device, &tetris.base.surface_config);
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
