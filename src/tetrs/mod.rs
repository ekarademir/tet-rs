use std::time::{self, Instant};

use anyhow::Context;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::Window,
};

use game_state::GameState;
use scene::{Frame, Scene};

pub use game_state::GameEvent;

const DELTA: u64 = 17;

pub struct Tetrs {
    game_state: GameState,
    scene: Scene,
    event_loop: EventLoopProxy<GameEvent>,
    last_stepped: time::Instant,
    debug_msg: String,
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
            debug_msg: String::new(),
        })
    }

    pub fn handle_up(&mut self) {
        self.game_state.tetromino_rotate();
        self.render().unwrap();
    }

    pub fn handle_down(&mut self) {
        self.game_state.tetromino_down();
        self.render().unwrap();
    }

    pub fn handle_right(&mut self) {
        self.game_state.tetromino_right();
        self.render().unwrap();
    }

    pub fn handle_left(&mut self) {
        self.game_state.tetromino_left();
        self.render().unwrap();
    }

    pub fn resize(&mut self, size: Frame) {
        self.scene.resize(&size);
    }

    pub fn step_time(&mut self) -> anyhow::Result<()> {
        let delta = time::Duration::from_millis(DELTA);
        if self.last_stepped.elapsed() > delta {
            self.game_state.step_time(&self.event_loop)?;
            self.last_stepped = time::Instant::now();
        }
        Ok(())
    }

    pub fn set_debug(&mut self, msg: String) {
        self.debug_msg = msg;
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let frame = self.scene.get_next_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.render_game(&view);
        self.scene.render_blocks(&view, &self.game_state);
        self.scene.render_next_tetromino(&view, &self.game_state);
        self.scene.render_current_tetromino(&view, &self.game_state);
        self.scene.render_score(&view, &self.game_state);
        self.scene.render_level(&view, &self.game_state);
        self.scene.render_debug(&view, &self.debug_msg);

        frame.present();
        Ok(())
    }
}

pub async fn run(window: Window, event_loop: EventLoop<GameEvent>, mut tetrs: Tetrs) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        tetrs.step_time().unwrap();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    tetrs.resize(size);
                    window.request_redraw();
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_code {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    VirtualKeyCode::Up => tetrs.handle_up(),
                    VirtualKeyCode::Down => tetrs.handle_down(),
                    VirtualKeyCode::Left => tetrs.handle_left(),
                    VirtualKeyCode::Right => tetrs.handle_right(),
                    _ => {}
                },
                _ => {}
            },
            Event::RedrawRequested(_) => {
                tetrs.render().unwrap();
            }
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
mod tetromino;
mod vertex;
mod writer;
