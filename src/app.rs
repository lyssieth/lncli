use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    data::{Data, LN},
    scrape::{self, Search},
    state::State,
    Res,
};

use cursive::{
    align::{Align, HAlign},
    direction::Orientation,
    event::Key,
    theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme},
    traits::{Nameable, Resizable, Scrollable},
    view::Margins,
    views::{
        DummyView, EditView, LinearLayout, OnEventView, PaddedView, Panel, SelectView, TextView,
        ThemedView,
    },
    Cursive, CursiveExt,
};
use log::{info, LevelFilter};
use owo_colors::OwoColorize;
use parking_lot::RwLock;

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

        t.palette[PaletteColor::Secondary] = Color::Light(BaseColor::Black);
        t.palette[PaletteColor::Highlight] = Color::Dark(BaseColor::Blue);
        t.palette[PaletteColor::HighlightText] = Color::Dark(BaseColor::Black);
        t.palette[PaletteColor::HighlightInactive] = Color::Light(BaseColor::Black);

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

    siv.add_active_screen();

    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('D', Cursive::toggle_debug_console);

    home_view(siv, &None);

    siv.run_crossterm()?;

    Ok(())
}

fn reader_view(siv: &mut Cursive) {
    info!("reader view");

    let state = siv.take_user_data();

    if state.is_none() {
        siv.pop_layer();
        home_view(siv, &None);
        error_panel(
            siv,
            "Nothing is configured to be read. Please use `s`, or select from the home screen.",
        );
        return;
    }

    let state: State = state.unwrap();
    {
        let state = state.clone();
        siv.set_user_data(state);
    }

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
                (&state.title).green(),
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
                "{q}uit, {c}hapter select, {h}ome view, {s}earch, and {arrow_keys}",
                c = "c".yellow(),
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
        .on_event('c', move |siv| {
            select_chapter(siv, &s1.clone());
        })
        .on_event(Key::Right, move |siv| {
            next_chapter(siv, &s2.clone());
        })
        .on_event(Key::Left, move |siv| {
            previous_chapter(siv, &state.clone());
        })
        .on_event('h', |siv| {
            home_view(siv, &None);
        });

    siv.pop_layer();
    siv.add_fullscreen_layer(layout);

    siv.clear_global_callbacks('r');
}

fn select_chapter(siv: &mut Cursive, state: &State) {
    info!("select chapter");

    let range = 1..=state.max_chapters;

    let state = state.clone();
    let chapter_select = EditView::new()
        .content(state.chapter.to_string())
        .max_content_width(5)
        .on_submit(move |s, text| {
            let text = text.trim();

            if text.is_empty() {
                s.pop_layer();
                error_panel(s, "please enter a chapter number"); // TODO: Nicer error messages
                return;
            }

            let chapter = text.parse::<usize>();

            if chapter.is_err() {
                s.pop_layer();
                error_panel(
                    s,
                    &format!(
                        "please enter a valid number ({})", // TODO: Nicer error messages
                        chapter.unwrap_err().to_string().red()
                    ),
                );
                return;
            }

            let chapter = chapter.unwrap();

            if !range.contains(&chapter) {
                s.pop_layer();
                error_panel(s, "please enter a number inside the range"); // TODO: Nicer error messages
                return;
            }

            let pat = format!("chapter-{}", state.chapter);
            let url = state
                .url
                .replace(pat.as_str(), &format!("chapter-{}", chapter));

            load_url(s, &url);

            s.pop_layer();
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
    info!("error panel");

    let layout = LinearLayout::vertical()
        .child(TextView::new(err).center())
        .child(TextView::new(format!("Press {esc} to close", esc = "esc".yellow())).center());

    let panel = Panel::new(layout).title("Error");
    let panel = OnEventView::new(panel).on_event(Key::Esc, |s| {
        s.pop_layer();
    });

    siv.add_layer(panel);
}

fn previous_chapter(siv: &mut Cursive, state: &State) {
    siv.pop_layer();

    if state.chapter == 1 {
        error_panel(siv, "You are already on the first chapter");
        return;
    }

    let url = state.url.replace(
        &format!("chapter-{}", state.chapter),
        &format!("chapter-{}", state.chapter - 1),
    );

    load_url(siv, &url);

    siv.pop_layer();
    reader_view(siv);
}

fn next_chapter(siv: &mut Cursive, state: &State) {
    info!("next chapter");

    if state.chapter == state.max_chapters {
        error_panel(siv, "You are already on the last chapter");
        return;
    }

    let url = state.url.replace(
        &format!("chapter-{}", state.chapter),
        &format!("chapter-{}", state.chapter + 1),
    );

    load_url(siv, &url);

    siv.pop_layer();
    reader_view(siv);
}

fn home_view(siv: &mut Cursive, updates: &Option<Vec<LN>>) {
    info!("home view");

    siv.pop_layer();

    siv.add_fullscreen_layer(DummyView);

    let data = Data::load();

    if data.is_err() {
        error_panel(siv, "Data missing/corrupted. Regenerating");
        let res = Data::new().save();

        assert!(res.is_ok(), "Failed to generate data. Please check that {:?} is accessible by your user. Error was: {}",
                Data::data_folder().yellow(), res.unwrap_err().yellow());
    }

    let data = data.unwrap();

    let mut main_view = LinearLayout::vertical();

    main_view.add_child(
        TextView::new(format!("{}", "Welcome to `lncli`".bold()))
            .center()
            .full_width(),
    );
    main_view.add_child(
        TextView::new(format!(
            "There are currently {} tracked novels",
            data.tracked().len().to_string().yellow()
        ))
        .center()
        .full_width(),
    );
    main_view.add_child(
        TextView::new(
            "The `update check` will also add an asterisk if you aren't caught up, so be aware.",
        )
        .center()
        .full_width(),
    );

    let submit = |s: &mut Cursive, novel: &LN| {
        let ch = format!("/chapter-{}.html", novel.last_chapter);
        load_url(s, &novel.url.replace(".html", &ch));
        reader_view(s);
    };

    let tv = {
        let mut sv = SelectView::new();
        for x in data.tracked() {
            if let Some(updates) = &updates {
                if updates.iter().any(|u| u.name == x.name) {
                    sv.add_item(format!("* {} ({})", &x.name, x.last_chapter), x.clone());
                } else {
                    sv.add_item(format!("{} ({})", &x.name, x.last_chapter), x.clone());
                }
            } else {
                sv.add_item(format!("{} ({})", &x.name, x.last_chapter), x.clone());
            }
        }

        sv.set_on_submit(submit);

        sv.h_align(HAlign::Center)
    };

    let tracked_panel = Panel::new(tv).title("Tracked Novels").full_screen();

    let rv = {
        let mut sv = SelectView::new();
        for x in data.recent().iter() {
            sv.add_item(format!("{} ({})", &x.name, x.last_chapter), x.clone());
        }

        sv.set_on_submit(submit);

        sv.h_align(HAlign::Center)
    };

    let recent_panel = Panel::new(rv).title("Recent Novels").full_screen();

    main_view.add_child(
        LinearLayout::horizontal()
            .child(tracked_panel)
            .child(recent_panel),
    );

    main_view.add_child(
        TextView::new(format!(
            "{q}uit, {r}eader, {s}earch, {u}pdate check, {enter} to select, {arrow_keys} to navigate",
            q = "q".yellow(),
            r = "r".yellow(),
            s = "s".yellow(),
            u = "u".yellow(),
            enter = "enter".yellow(),
            arrow_keys = "arrow keys".yellow()
        ))
        .align(Align::bot_right()),
    );

    let main_view = OnEventView::new(main_view)
        .on_event('r', reader_view)
        .on_event('s', |s| {
            search_view(s, None);
        })
        .on_event('u', |s| {
            update_check(s);
        });

    siv.add_fullscreen_layer(main_view.full_height());
}

fn update_check(cursive: &mut Cursive) {
    info!("update check");

    let data = Data::load();

    if data.is_err() {
        error_panel(cursive, "Failed to load data");
        return;
    }

    let data = data.unwrap();

    let mut found = Vec::new();

    for x in data.tracked() {
        let check = scrape::update_check(&x.url, x.last_chapter);

        if let Ok(check) = check {
            if check {
                found.push(x.clone());
            }
        } else {
            error_panel(
                cursive,
                &format!("Failed to check for updates: {}", check.unwrap_err()),
            );
        }
    }

    home_view(cursive, &Some(found));
}

fn load_url(siv: &mut Cursive, url: &str) {
    info!("LOAD_URL: Attempting to load: {}", url.green());
    let output = scrape::load(url);

    if let Err(e) = output {
        error_panel(siv, &format!("{}", e.to_string().red()));
        panic!("{}", e);
    }

    let output = output.unwrap();

    let state = State::from_output(url, output.clone());

    info!(
        "LOAD_URL: Successfully loaded state from url {}",
        &state.url,
    );

    let data = Data::load();

    let mut data = if let Err(e) = data {
        if e.to_string().starts_with("data file does not exist") {
            Data::new()
        } else {
            panic!("{}", e);
        }
    } else {
        data.unwrap()
    };

    data.recent_mut().push_front(LN {
        name: output.name.clone(),
        url: url.to_owned(),
        last_chapter: output.chapter,
    });

    let test_url = url.split_at(url.find("chapter-").unwrap()).0;
    let test_url = test_url[0..test_url.len() - 1].to_owned();
    let test_url = test_url + ".html";

    info!("test_url is: {}", test_url);

    let test_url = &test_url;

    if data.tracked().iter().any(|x| x.url.starts_with(test_url)) {
        info!("found tracked novel, updating latest chapter read");

        data.tracked_mut()
            .iter_mut()
            .filter(|x| x.url.starts_with(test_url))
            .for_each(|x| {
                x.last_chapter = output.chapter;
            });
    }

    let save_res = data.save();

    if let Err(e) = save_res {
        panic!("{}", e);
    }

    siv.set_user_data(state);
}

fn search_view(siv: &mut Cursive, results: Option<Search>) {
    let results_mode = results.is_some();
    let search_box = {
        let ev = EditView::new().on_submit(move |s, text| {
            let text = text.trim();

            if text.is_empty() {
                error_panel(s, "please enter a search term");
                return;
            }

            search_url(s, text);
        });

        if let Some(results) = &results {
            ev.content(&results.query)
        } else {
            ev
        }
    };

    let search_results = { results.as_ref().map(create_sv) };

    let search_layout = {
        let mut layout = LinearLayout::vertical();

        if let Some(search_results) = search_results {
            let query = results.unwrap().query;
            let query = query.trim();
            let query = "        ".to_owned() + query;

            layout.add_child(
                TextView::new(query.green().to_string())
                    .h_align(HAlign::Center)
                    .no_wrap(),
            );
            layout.add_child(
                TextView::new("-".repeat(query.len()))
                    .h_align(HAlign::Center)
                    .no_wrap(),
            );

            layout.add_child(search_results);
        } else {
            layout.add_child(search_box);
        }

        layout
    };

    let results_mode_text = {
        if results_mode {
            format!(
                "{t}rack, {esc} to go back, {enter} to select, {arrow_keys} to navigate",
                t = "t".yellow(),
                esc = "esc".yellow(),
                enter = "enter".yellow(),
                arrow_keys = "arrow keys".yellow()
            )
        } else {
            format!(
                "{esc} to go back, {enter} to search",
                esc = "esc".yellow(),
                enter = "Enter".yellow()
            )
        }
    };

    let controls = TextView::new(results_mode_text).align(Align::bot_right());

    let layout = LinearLayout::new(Orientation::Vertical)
        .child(search_layout)
        .child(controls);

    let panel = Panel::new(layout).title(if results_mode {
        "Search Results"
    } else {
        "Search"
    });

    let view = OnEventView::new(panel).on_event(Key::Esc, |s| {
        s.pop_layer();
        s.pop_layer();

        home_view(s, &None);
    });

    siv.add_layer(view);
}

fn create_sv(search: &Search) -> cursive::views::OnEventView<cursive::views::SelectView<url::Url>> {
    let mut sv = SelectView::new().h_align(HAlign::Center);

    if search.results.is_empty() {
        sv.set_on_submit(|s, _| {
            s.pop_layer();
            error_panel(s, "No results found.\nTry a shorter query.");
        });
        return OnEventView::new(sv);
    }

    for (url, label) in search.results.clone() {
        sv.add_item(label, url);
    }

    sv.set_on_submit(|s, url| {
        let url = url.as_str();

        let url = url.replace(".html", "/chapter-1.html");

        load_url(s, &url);
        s.pop_layer();
        reader_view(s);
    });

    let selected = Arc::new(RwLock::new(Some(search.results[0].0.to_string())));

    let si = selected.clone();
    sv.set_on_select(move |_, url| {
        let url = url.as_str();

        si.write_arc().replace(url.to_owned());
    });

    OnEventView::new(sv).on_event('t', move |s| {
        let selected = selected.read_arc();
        if selected.is_none() {
            info!("Selected was none");
            return;
        }
        let sel = selected.clone().unwrap();

        info!("Selected: {}", sel.green());

        let data = Data::load();

        if let Ok(mut data) = data {
            data.tracked_mut().push(LN {
                name: scrape::get_name(&sel).unwrap(),
                url: sel.clone(),
                last_chapter: 1,
            });

            let data = data.save();

            if let Err(e) = data {
                error_panel(s, &e.to_string());
            }
        }
    })
}

fn search_url(siv: &mut Cursive, query: &str) {
    info!("search query: {}", query.green());

    let output = scrape::search(query);

    siv.pop_layer();
    if let Err(e) = &output {
        search_view(siv, None);
        error_panel(siv, &format!("{}", e.to_string().red()));
        return;
    }

    let output = output.unwrap();

    search_view(siv, Some(output));
}
