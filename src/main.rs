#![warn(clippy::pedantic, clippy::nursery)]

use color_eyre::Report;

type Res<T> = Result<T, Report>;

mod app;
mod data;
mod scrape;
mod state;

fn main() -> Res<()> {
    color_eyre::install()?;

    app::run()?;

    Ok(())
}
