mod tetrs;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .context("Couldn't initialise the logger")?;

    pollster::block_on(execute())?;

    Ok(())
}

async fn execute() -> anyhow::Result<()> {
    let event_loop =
        winit::event_loop::EventLoopBuilder::<tetrs::GameEvent>::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::LogicalSize::new(600, 600))
        .with_title("Tetrs")
        .build(&event_loop)
        .context("Couldn't initialise the window")?;

    let tetrs = tetrs::Tetrs::new(&window, &event_loop)
        .await
        .context("Can't create tetrs")?;

    Ok(tetrs::run(window, event_loop, tetrs).await)
}
