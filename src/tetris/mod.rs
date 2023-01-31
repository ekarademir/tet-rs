mod base;
mod colours;
mod drawable;
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
use scene::{Frame, Scene};
use vertex::Vertex;

pub struct Tetris {
    base: Base,
    game_state: GameState,
    scene: Scene,
}

impl Tetris {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Tetris> {
        let base = Base::new(window)
            .await
            .context("Couldn't initialize base")?;
        let game_state = GameState::default();
        let scene = Scene::new(&base);

        Ok(Tetris {
            base,
            game_state,
            scene,
        })
    }

    pub fn resize(&mut self, size: Frame) {
        self.base.surface_config.width = size.width;
        self.base.surface_config.height = size.height;
        self.scene.resize(&size);
        self.base
            .surface
            .configure(&self.base.device, &self.base.surface_config);
    }

    pub fn render_all(&self) {
        let frame = self.get_next_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.render_game_area(&self, &view);

        frame.present();
    }

    fn get_next_frame(&self) -> wgpu::SurfaceTexture {
        self.base
            .surface
            .get_current_texture()
            .expect("Couldn't get next swapchain texture")
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
                tetris.render_all();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
