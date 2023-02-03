mod base;
mod colours;
mod drawable;
mod game_state;
mod scene;
mod vertex;
mod writer;

use anyhow::Context;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use base::Base;
use game_state::GameState;
use scene::{Frame, Scene};
use vertex::Vertex;
use writer::Writer;

pub struct Tetrs {
    base: Base,
    game_state: GameState,
    scene: Scene,
}

impl Tetrs {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Tetrs> {
        let base = Base::new(window)
            .await
            .context("Couldn't initialize base")?;
        let game_state = GameState::default();
        let scene = Scene::new(&base).context("Couldn't create the scene")?;

        Ok(Tetrs {
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

    pub fn render_all(&mut self) -> anyhow::Result<()> {
        let frame = self.get_next_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.render_game(&self.base, &view);
        self.scene
            .render_blocks(&self.base, &view, &self.game_state);
        self.scene.write(
            &self.base,
            &view,
            "next\n\n\n\n\n\nscore   1234\n\nlevel   12",
        );

        frame.present();
        Ok(())
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
    mut tetrs: Tetrs,
) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                tetrs.resize(size);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                tetrs.render_all().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
