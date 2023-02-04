use std::time::{self, Instant};

use anyhow::Context;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::Window,
};

use game_state::GameState;
use scene::{Frame, Scene};

pub use game_state::GameEvent;

pub struct Tetrs {
    game_state: GameState,
    scene: Scene,
    event_loop: EventLoopProxy<GameEvent>,
    last_stepped: time::Instant,
}

impl Tetrs {
    pub async fn new(window: &Window, event_loop: &EventLoop<GameEvent>) -> anyhow::Result<Tetrs> {
        let game_state = GameState::default();
        let scene = Scene::new(window)
            .await
            .context("Couldn't create the scene")?;

        let event_loop = event_loop.create_proxy();

        Ok(Tetrs {
            game_state,
            scene,
            event_loop,
            last_stepped: Instant::now(),
        })
    }

    pub fn resize(&mut self, size: Frame) {
        self.scene.resize(&size);
    }

    pub fn step_time(&mut self) -> anyhow::Result<()> {
        let delta = time::Duration::from_millis(250);
        if self.last_stepped.elapsed() > delta {
            self.game_state.step_time(&self.event_loop)?;
            self.last_stepped = time::Instant::now();
        }
        Ok(())
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

pub async fn run(window: Window, event_loop: EventLoop<GameEvent>, mut tetrs: Tetrs) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        tetrs.step_time().unwrap();

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
            Event::UserEvent(GameEvent::Step) => {
                tetrs.render().unwrap();
            }
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
