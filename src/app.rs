use std::path::PathBuf;

use cursive::{
    align::{Align, HAlign},
    direction::Orientation,
    event::Key,
    theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme},
    traits::{Nameable, Resizable, Scrollable},
    view::Margins,
    views::{
        DummyView, EditView, LinearLayout, OnEventView, PaddedView, Panel, TextView, ThemedView,
    },
    Cursive, CursiveExt,
};
use log::{error, info, LevelFilter};
use owo_colors::OwoColorize;

use crate::{
    data::{Data, LN},
    scrape,
    state::State,
    Res,
};

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

pub fn run() -> Res<()> {
    let mut cursive = cursive::crossterm();

    cursive::logger::init();

    log::set_max_level(LevelFilter::Info);

    let siv = &mut cursive;
    {
        let theme = get_theme();
        siv.set_theme(theme);
    }

    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('D', Cursive::toggle_debug_console);
    siv.add_global_callback('s', search_view);

    home_view(siv);

    {
        // TODO: REMOVE THIS DEBUG LOAD AND LOAD PROPERLY YA DAFT CUNT
        load_url(
            siv,
            "https://freewebnovel.com/prime-originator/chapter-69.html",
            false,
        );
    }

    siv.run_crossterm()?;

    Ok(())
}

fn reader_view(siv: &mut Cursive) {
    info!("reader view");
    siv.pop_layer();

    let state = siv.take_user_data();

    if state.is_none() {
        home_view(siv);
        return;
    }

    let state: State = state.unwrap();

    let size = siv.screen_size();

    let main_content = TextView::new(&state.content)
        .h_align(HAlign::Center)
        .full_height();
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
                state.title.green(),
                state.chapter.yellow(),
                state.max_chapters.yellow()
            ))
            .center()
            .fixed_height(2)
            .with_name("title"),
        )
        .child(PaddedView::new(margins, main_content.scrollable()).with_name("main_content"))
        .child(
            TextView::new(format!(
                "press {q} to quit, {C} to select chapter by number, {h} to go home, {s} to search, and {arrow_keys} to navigate",
                C = "C".yellow(),
                q = "q".yellow(),
                h = "h".yellow(),
                s = "s".yellow(),
                arrow_keys = "arrow keys".yellow()
            ))
            .align(Align::bot_right())
            .with_name("footer"),
        );

    let s1 = state.clone();
    let s2 = state.clone();

    let layout = OnEventView::new(layout)
        .on_event('C', move |siv| {
            select_chapter(siv, &s1.clone());
        })
        .on_event(Key::Right, move |siv| {
            next_chapter(siv, &s2.clone());
        })
        .on_event(Key::Left, move |siv| {
            previous_chapter(siv, &state.clone());
        });

    siv.add_fullscreen_layer(layout);

    siv.clear_global_callbacks('r');
    siv.add_global_callback('h', home_view);
}

fn select_chapter(siv: &mut Cursive, state: &State) {
    info!("select chapter");

    let range = 1..=state.max_chapters;

    let chapter_select = EditView::new()
        .content(state.chapter.to_string())
        .max_content_width(5)
        .on_submit(move |s, text| {
            s.pop_layer();
            let text = text.trim();

            if text.is_empty() {
                error_panel(s, "please enter a chapter number");
                return;
            }

            let chapter = text.parse::<usize>();

            if chapter.is_err() {
                error_panel(
                    s,
                    &format!(
                        "please enter a valid number ({})",
                        chapter.unwrap_err().to_string().red()
                    ),
                );
                return;
            }

            let chapter = chapter.unwrap();

            if !range.contains(&chapter) {
                error_panel(s, "please enter a number inside the range");
                return;
            }

            load_url(
                s,
                &format!(
                    "https://freewebnovel.com/prime-originator/chapter-{}.html",
                    chapter
                ),
                false,
            );

            reader_view(s);
        });

    let layout = LinearLayout::vertical()
        .child(chapter_select)
        .child(TextView::new(format!(
            "Enter a number between {} and {}.\nAnd press {enter}",
            1.yellow(),
            state.max_chapters.yellow(),
            enter = "Enter".yellow()
        )));
    let panel = Panel::new(layout).title("Chapter Select");

    siv.add_layer(panel);
}

fn error_panel(siv: &mut Cursive, err: &str) {
    siv.pop_layer();
    info!("error panel");

    let layout = LinearLayout::vertical()
        .child(TextView::new(err).center())
        .child(TextView::new("Press esc to pop layer").center());

    let panel = Panel::new(layout).title("Error");
    let panel = OnEventView::new(panel).on_event(Key::Esc, |s| {
        s.pop_layer();
    });

    siv.add_layer(panel);
}

fn previous_chapter(siv: &mut Cursive, state: &State) {
    siv.pop_layer();

    let url = state.url.replace(
        &format!("chapter-{}", state.chapter),
        &format!("chapter-{}", state.chapter - 1),
    );

    load_url(siv, &url, false);

    reader_view(siv);
}

fn next_chapter(siv: &mut Cursive, state: &State) {
    info!("next chapter");
    siv.pop_layer();

    let url = state.url.replace(
        &format!("chapter-{}", state.chapter),
        &format!("chapter-{}", state.chapter + 1),
    );

    load_url(siv, &url, false);

    reader_view(siv);
}

fn home_view(siv: &mut Cursive) {
    info!("home view");
    siv.pop_layer();

    siv.clear_global_callbacks('h');

    siv.add_fullscreen_layer(DummyView); // TODO: Write the home view

    siv.add_global_callback('r', reader_view);
}

fn load_url(siv: &mut Cursive, url: &str, should_pop: bool) {
    if should_pop {
        siv.pop_layer();
    }
    info!("LOAD_URL: Attempting to load: {}", url.green());
    let output = scrape::load(url);

    if let Err(e) = output {
        error_panel(siv, &format!("{}", e.to_string().red()));
        return;
    }

    let output = output.unwrap();

    let state = State::from_output(url, output.clone());

    info!(
        "LOAD_URL: Successfully loaded state from url {}",
        &state.url
    );

    let data = Data::load();

    let mut data = if let Err(e) = data {
        if e.to_string().starts_with("data file does not exist") {
            Data::new()
        } else {
            error!("{}", e);
            return;
        }
    } else {
        data.unwrap()
    };

    data.recent_mut().push(LN {
        name: output.name.clone(),
        url: url.to_owned(),
        last_chapter: output.chapter,
    });

    let save_res = data.save();

    if let Err(e) = save_res {
        error!("{}", e);
        return;
    }

    siv.set_user_data(state);
}

fn search_view(siv: &mut Cursive) {
    info!("search view");
    let layout = LinearLayout::new(Orientation::Vertical);

    let panel = Panel::new(layout)
        .title("Search")
        .title_position(HAlign::Left);

    let view = OnEventView::new(panel)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event('s', |s| {
            s.pop_layer();
        });

    siv.add_layer(view);
}
