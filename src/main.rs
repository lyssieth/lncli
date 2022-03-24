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
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use lncli::{App, Res};
struct State {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub app: App,
}

impl State {
    pub fn new(mut stdout: Stdout) -> Res<Self> {
        enable_raw_mode()?;

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            app: App::new(),
        })
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
            .split(f.size());

        let title = self.app.get_title();
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
