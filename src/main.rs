#![warn(clippy::pedantic)]

use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::time::sleep;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use lncli::Res;
struct State {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl State {
    pub fn new(mut stdout: Stdout) -> Res<Self> {
        enable_raw_mode()?;

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }
}

impl Drop for State {
    fn drop(&mut self) {
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        disable_raw_mode().unwrap();
    }
}

#[tokio::main]
async fn main() -> Res<()> {
    color_eyre::install()?;

    let mut state = State::new(io::stdout())?;

    state.terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::all());
        f.render_widget(block, size);
    })?;

    sleep(Duration::from_millis(5000)).await;

    Ok(())
}
