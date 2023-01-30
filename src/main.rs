mod tetris;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .context("Couldn't initialise the logger")?;
    log::info!("Hello, world!");

    pollster::block_on(execute())?;

    Ok(())
}

async fn execute() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window =
        winit::window::Window::new(&event_loop).context("Couldn't initialise the window")?;
    let tetris = tetris::Tetris::new(&window)
        .await
        .context("Can't create tetris")?;

    Ok(tetris::run(window, event_loop, tetris).await)
}
