use std::time::{self, Instant};

use anyhow::Context;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    monitor::VideoMode,
    window::{Fullscreen, Window},
};

use game_state::GameState;
use scene::{Frame, Scene};

const DELTA: u64 = 17;

#[derive(PartialEq)]
enum TetrsState {
    Bootstrapped,
    Running,
    Paused,
    Finished,
}

#[derive(Debug)]
pub enum GameEvent {
    Step,
    Pause,
    Fullscreen,
    Finished,
}

pub struct Tetrs {
    game_state: GameState,
    scene: Scene,
    event_loop: EventLoopProxy<GameEvent>,
    last_stepped: time::Instant,
    debug_msg: String,
    state: TetrsState,
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
            state: TetrsState::Bootstrapped,
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
        if self.state == TetrsState::Bootstrapped {
            self.state = TetrsState::Running;
            self.event_loop.send_event(GameEvent::Fullscreen)?;
        }
        if self.state == TetrsState::Running {
            let delta = time::Duration::from_millis(DELTA);
            if self.last_stepped.elapsed() > delta {
                self.game_state.step_time(&self.event_loop)?;
                self.last_stepped = time::Instant::now();
            }
        }
        Ok(())
    }

    pub fn set_debug(&mut self, msg: String) {
        self.debug_msg = msg;
    }

    pub fn toggle_pause(&mut self) -> anyhow::Result<()> {
        if self.state == TetrsState::Running {
            self.state = TetrsState::Paused;
            self.event_loop.send_event(GameEvent::Pause)?;
        } else {
            self.state = TetrsState::Running;
        }
        Ok(())
    }

    pub fn finish_game(&mut self) -> anyhow::Result<()> {
        self.state = TetrsState::Finished;
        self.render().context("Can't render after finish")?;
        Ok(())
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let frame = self.scene.get_next_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if self.state != TetrsState::Finished {
            self.scene.render_game(&view);
            self.scene.render_blocks(&view, &self.game_state);
            self.scene.render_next_tetromino(&view, &self.game_state);
            self.scene.render_current_tetromino(&view, &self.game_state);
            self.scene.render_score(&view, &self.game_state);
            self.scene.render_level(&view, &self.game_state);
            self.scene.render_debug(&view, &self.debug_msg);
            if self.state == TetrsState::Paused {
                self.scene.render_pause(&view, &self.game_state);
            }
        } else {
            self.scene.render_finish_screen(&view, &self.game_state);
        }

        frame.present();
        Ok(())
    }
}

fn toggle_fullscreen(window: &Window, mode: &VideoMode) {
    if window.fullscreen().is_some() {
        window.set_fullscreen(None);
        window.set_inner_size(winit::dpi::LogicalSize::new(
            super::WINDOW_WIDTH,
            super::WINDOW_HEIGHT,
        ));
        window.request_redraw();
    } else {
        let fullscreen = Some(Fullscreen::Exclusive(mode.clone()));
        window.set_fullscreen(fullscreen);
        window.request_redraw();
    }
}

pub async fn run(
    window: Window,
    event_loop: EventLoop<GameEvent>,
    mut tetrs: Tetrs,
) -> anyhow::Result<()> {
    // Machinery for the full screen mode
    let monitor = event_loop
        .available_monitors()
        .next()
        .context("Can't find a monitor")?;
    let modes: Vec<_> = monitor.video_modes().collect();
    if modes.len() == 0 {
        return Err(anyhow::Error::msg("Can't find a mode for fullscreen"));
    }
    let maybe_mode = {
        let mut max_mode_idx: usize = 0;
        let mut max_size = winit::dpi::PhysicalSize::new(0, 0);
        for (mode_idx, mode) in modes.iter().enumerate() {
            if mode.size().width > max_size.width && mode.size().height > max_size.height {
                max_size = mode.size();
                max_mode_idx = mode_idx;
            }
        }

        monitor.video_modes().nth(max_mode_idx)
    };

    let fullscreen_mode = maybe_mode.context("Can't obtain max size mode for fullscreen")?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        tetrs.step_time().expect("Panicked while stepping time");

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
                    VirtualKeyCode::Space => {
                        tetrs.toggle_pause().expect("Panicked while toggling pause")
                    }
                    VirtualKeyCode::F => {
                        toggle_fullscreen(&window, &fullscreen_mode);
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::RedrawRequested(_) => {
                tetrs.render().expect("Panicked while render");
            }
            Event::UserEvent(GameEvent::Step | GameEvent::Pause) => {
                tetrs.render().expect("Panicked while render");
            }
            Event::UserEvent(GameEvent::Fullscreen) => {
                toggle_fullscreen(&window, &fullscreen_mode);
            }
            Event::UserEvent(GameEvent::Finished) => {
                tetrs.finish_game().unwrap();
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
