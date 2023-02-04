use std::fmt::format;

use anyhow::Context;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use game_state::GameState;
use scene::{Frame, Scene};

pub struct Tetrs {
    game_state: GameState,
    scene: Scene,
}

impl Tetrs {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Tetrs> {
        let game_state = GameState::default();
        let scene = Scene::new(window)
            .await
            .context("Couldn't create the scene")?;

        Ok(Tetrs { game_state, scene })
    }

    pub fn resize(&mut self, size: Frame) {
        self.scene.resize(&size);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let frame = self.scene.get_next_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.render_game(&view);
        self.scene.render_blocks(&view, &self.game_state);
        self.scene.render_next(&view, &self.game_state);
        self.scene.render_score(&view, &self.game_state);
        self.scene.render_level(&view, &self.game_state);
        self.scene
            .render_debug(&view, format!("{:?}", self.game_state.time_elapsed));

        frame.present();
        Ok(())
    }
}

pub async fn run(
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    mut tetrs: Tetrs,
) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        tetrs.game_state.step_time();

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                tetrs.resize(size);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                tetrs.render().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

mod base;
mod colours;
mod drawable;
mod game_state;
mod scene;
mod vertex;
mod writer;
