#![warn(clippy::pedantic)]

use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

use color_eyre::{eyre::bail, Report};
use cursive::{views::TextView, Cursive, CursiveExt, CursiveRunnable};

type Res<T> = Result<T, Report>;

mod scrape;

struct State {
    cursive: CursiveRunnable,
}

impl State {
    pub fn new() -> Self {
        let cursive = cursive::crossterm();

        Self { cursive }
    }

    pub async fn run(&mut self) -> Res<()> {
        let siv = &mut self.cursive;

        siv.add_layer(TextView::new("Hello, world!"));

        siv.add_global_callback('q', Cursive::quit);

        siv.run_crossterm()?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Res<()> {
    color_eyre::install()?;

    let mut state = State::new();

    let path: PathBuf = "theme.toml".into();

    if path.exists() {
        let res = state.cursive.load_theme_file(path);

        if let Err(err) = res {
            bail!("{:?}", err);
        }
    }

    state.run().await?;

    Ok(())
}
