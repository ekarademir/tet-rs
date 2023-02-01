mod tetris;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .context("Couldn't initialise the logger")?;

    // let t = tetris::text::Text::new();
    // log::debug!("{:?}", t);

    pollster::block_on(execute())?;

    Ok(())
}

async fn execute() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::LogicalSize::new(600, 600))
        .with_title("Tetrs")
        .build(&event_loop)
        .context("Couldn't initialise the window")?;

    let tetris = tetris::Tetrs::new(&window)
        .await
        .context("Can't create tetris")?;

    Ok(tetris::run(window, event_loop, tetris).await)
}
