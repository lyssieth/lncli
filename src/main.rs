#![warn(clippy::pedantic)]

use std::path::PathBuf;

use color_eyre::Report;
use cursive::{
    align::{Align, HAlign},
    event::Key,
    theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme},
    traits::{Nameable, Resizable, Scrollable},
    utils::markup::markdown::parse,
    view::Margins,
    views::{DummyView, LinearLayout, OnEventView, PaddedView, Panel, TextView, ThemedView},
    Cursive, CursiveExt, CursiveRunnable,
};
use log::{error, info};
use owo_colors::OwoColorize;
use scrape::Output;
use serde::{Deserialize, Serialize};

type Res<T> = Result<T, Report>;

mod scrape;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LN {
    name: String,
    url: String,
    last_chapter: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    tracked_novels: Vec<LN>,
    recent_novels: Vec<LN>,
}

#[derive(Debug, Clone)]
struct State {
    url: String,
    title: String,
    chapter: usize,
    max_chapters: usize,
    content: String,
}

impl State {
    fn from_url(url: &str) -> Res<Self> {
        let Output {
            title,
            content,
            chapter,
            max_chapters,
        } = scrape::load(url)?;
        Ok(Self {
            url: url.to_string(),
            title,
            chapter,
            max_chapters,
            content,
        })
    }
}
struct App {
    cursive: CursiveRunnable,
}

impl App {
    pub fn new() -> Self {
        let cursive = cursive::crossterm();

        cursive::logger::init();

        Self { cursive }
    }

    pub fn run(&mut self) -> Res<()> {
        let siv = &mut self.cursive;
        {
            let theme = get_theme();
            siv.set_theme(theme);
        }

        siv.add_global_callback('q', Cursive::quit);
        siv.add_global_callback('d', Cursive::toggle_debug_console);
        siv.add_global_callback('s', Self::search_view);

        Self::home_view(siv);

        siv.run_crossterm()?;

        Ok(())
    }

    fn reader_view(siv: &mut Cursive) {
        siv.pop_layer();

        let state = siv.take_user_data();

        if state.is_none() {
            Self::home_view(siv);
            return;
        }

        let State {
            title,
            chapter,
            max_chapters,
            content,
            ..
        } = state.unwrap();

        let size = siv.screen_size();

        let main_content = TextView::new(content).h_align(HAlign::Center).full_height();
        let main_content = ThemedView::new(get_theme(), main_content);
        let margins = Margins {
            left: if size.x == 0 { 8 } else { size.x / 8 },
            right: if size.x == 0 { 8 } else { size.x / 8 },
            top: 0,
            bottom: 0,
        };

        let layout = LinearLayout::vertical()
            .child(
                TextView::new(format!(
                    "{} - Chapter {}/{}",
                    title.green(),
                    chapter.yellow(),
                    max_chapters.yellow()
                ))
                .center()
                .fixed_height(2)
                .with_name("title"),
            )
            .child(PaddedView::new(margins, main_content.scrollable()).with_name("main_content"))
            .child(
                TextView::new(format!(
                    "press {q} to quit, {s} to search, and {arrow_keys} to navigate",
                    q = "q".yellow(),
                    s = "s".yellow(),
                    arrow_keys = "arrow keys".yellow()
                ))
                .align(Align::bot_right())
                .with_name("footer"),
            );

        let layout = OnEventView::new(layout)
            .on_event(Key::Right, Self::next_chapter)
            .on_event(Key::Left, Self::previous_chapter);

        siv.add_fullscreen_layer(layout);

        siv.clear_global_callbacks('r');
        siv.add_global_callback('h', Self::home_view);
    }

    fn previous_chapter(_siv: &mut Cursive) {
        todo!("Load the previous chapter");
    }

    fn next_chapter(_siv: &mut Cursive) {
        todo!("Load the next chapter");
    }

    fn home_view(siv: &mut Cursive) {
        siv.pop_layer();

        siv.add_fullscreen_layer(DummyView);

        let state = State::from_url("https://freewebnovel.com/prime-originator/chapter-69.html");

        if let Ok(state) = state {
            info!("Successfully loaded state from url {}", &state.url);
            siv.set_user_data(state);
        } else {
            error!("{}", state.unwrap_err());
        }

        siv.add_global_callback('r', Self::reader_view);

        // todo!("Write the home view");
    }

    fn search_view(siv: &mut Cursive) {
        let markdown = parse("Hello, world!\n\nI am a *wonderful* **bean**!");

        let inner = TextView::new(markdown).center();

        let view = OnEventView::new(inner)
            .on_event(Key::Esc, |s| {
                s.pop_layer();
            })
            .on_event('s', |s| {
                s.pop_layer();
            });

        let panel = Panel::new(view)
            .title("Search")
            .title_position(HAlign::Left);

        siv.add_layer(panel);
    }
}

fn main() -> Res<()> {
    color_eyre::install()?;

    let mut state = App::new();

    state.run()?;

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
