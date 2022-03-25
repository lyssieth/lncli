#![warn(clippy::pedantic)]

use std::path::PathBuf;

use color_eyre::Report;
use cursive::{
    align::{Align, HAlign},
    theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme},
    traits::{Resizable, Scrollable},
    view::Margins,
    views::{LinearLayout, PaddedView, TextView, ThemedView},
    Cursive, CursiveExt, CursiveRunnable,
};
use owo_colors::OwoColorize;

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
        {
            let theme = get_theme();
            siv.set_theme(dbg!(theme));
        }

        let size = siv.screen_size();

        let main_content = TextView::new(lipsum::lipsum(2000))
            .h_align(HAlign::Center)
            .full_height();
        let main_content = ThemedView::new(get_theme(), main_content);
        let margins = Margins {
            left: if size.x == 0 { 8 } else { size.x / 4 },
            right: if size.x == 0 { 8 } else { size.x / 4 },
            top: 0,
            bottom: 0,
        };

        let layout = LinearLayout::vertical()
            .child(
                TextView::new("Title and Shit Like qThat")
                    .center()
                    .fixed_height(2),
            )
            .child(PaddedView::new(margins, main_content.scrollable()))
            .child(
                TextView::new(format!(
                    "press {q} to quit, {s} to search, and {arrow_keys} to navigate",
                    q = "q".yellow(),
                    s = "s".yellow(),
                    arrow_keys = "arrow keys".yellow()
                ))
                .align(Align::bot_right()),
            );

        siv.add_fullscreen_layer(layout);

        siv.add_global_callback('q', Cursive::quit);

        siv.run_crossterm()?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Res<()> {
    color_eyre::install()?;

    let mut state = State::new();

    state.run().await?;

    Ok(())
}

fn get_theme() -> Theme {
    let path: PathBuf = "theme.toml".into();

    if path.exists() {
        cursive::theme::load_theme_file(path).expect("failed to load theme from `theme.toml`")
    } else {
        let mut t = Theme {
            shadow: false,
            borders: BorderStyle::Simple,
            palette: Palette::default(),
        };

        t.palette[PaletteColor::Background] = Color::Dark(BaseColor::Black);
        t.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
        t.palette[PaletteColor::Primary] = Color::Light(BaseColor::White);

        t
    }
}
