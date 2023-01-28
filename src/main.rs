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
    Ok(tetris::Tetris::new()
        .await
        .context("Can't create tetris")?
        .run()
        .await)
}
