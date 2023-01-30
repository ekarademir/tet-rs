mod base;
mod game_state;
mod scene;
mod vertex;

use anyhow::Context;
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

    pub fn render_all(&self, view: &wgpu::TextureView) {
        self.scene.render_game_scene(&self, view);
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

                tetris.render_all(&view);
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
