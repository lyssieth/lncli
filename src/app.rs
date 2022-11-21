use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    data::{Data, LN},
    scrape::{self, Search},
    state::State,
    Res,
};

use cursive::theme::{Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::views::NamedView;
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
use log::{error, info, LevelFilter};
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
        t.palette[PaletteColor::View] = Color::TerminalDefault;
        t.palette[PaletteColor::Primary] = Color::Light(BaseColor::White);

        t.palette[PaletteColor::Secondary] = Color::Light(BaseColor::Black);
        t.palette[PaletteColor::Highlight] = Color::Dark(BaseColor::Black);
        t.palette[PaletteColor::HighlightText] = Color::Light(BaseColor::White);
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
            TextView::new({
                let mut s = StyledString::new();

                s.append_styled(&state.title, Color::Dark(BaseColor::Green));
                s.append_plain(" - ");
                s.append_styled(format!("{}", state.chapter), Color::Dark(BaseColor::Yellow));
                s.append_plain("/");
                s.append_styled(
                    format!("{}", state.max_chapters),
                    Color::Dark(BaseColor::Yellow),
                );
                s
            })
            .center()
            .fixed_height(2)
            .with_name("title"),
        )
        .child(PaddedView::new(margins, main_content.scrollable()).with_name("main_content"))
        .child(
            TextView::new({
                let mut s = StyledString::new();

                s.append_styled("q", Color::Dark(BaseColor::Yellow));
                s.append_plain("uit, ");
                s.append_styled("c", Color::Dark(BaseColor::Yellow));
                s.append_plain("hapter select, ");
                s.append_styled("h", Color::Dark(BaseColor::Yellow));
                s.append_plain("ome view, ");
                s.append_styled("s", Color::Dark(BaseColor::Yellow));
                s.append_plain("earch, ");
                s.append_styled("arrow keys", Color::Dark(BaseColor::Yellow));
                s.append_plain(" to navigate");

                s
            })
            .align(Align::bot_right())
            .with_name("footer"),
        );

    let s1 = state.clone();
    let s2 = state.clone();
    let s3 = state.clone();

    let layout = OnEventView::new(layout)
        .on_event('c', move |siv| {
            select_chapter(siv, &s1.clone());
        })
        .on_event(Key::Right, move |siv| {
            next_chapter(siv, &s2.clone());
        })
        .on_event(Key::Left, move |siv| {
            previous_chapter(siv, &s3.clone());
        })
        .on_event('h', |siv| {
            home_view(siv, &None);
        })
        .on_event('O', move |siv| {
            let res = open::that(&state.url);

            if res.is_err() {
                error_panel(siv, "Could not open the url.");
                error!("Could not open the url: {}", res.err().unwrap());
            }
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
                        chapter.unwrap_err()
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
        .child(TextView::new({
            let mut s = StyledString::new();

            s.append_plain("Enter a number between ");
            s.append_styled("1", Color::Dark(BaseColor::Yellow));
            s.append_plain(" and ");
            s.append_styled(
                format!("{}", state.max_chapters),
                Color::Dark(BaseColor::Yellow),
            );
            s.append_plain(".\nAnd press ");
            s.append_styled("Enter", Color::Dark(BaseColor::Yellow));

            s
        }));
    let panel = Panel::new(layout).title("Chapter Select");

    siv.add_layer(panel);
}

fn error_panel(siv: &mut Cursive, err: &str) {
    info!("error panel");

    let layout = LinearLayout::vertical()
        .child(
            TextView::new({
                let mut s = StyledString::new();

                s.append_styled(err, Color::Dark(BaseColor::Red));

                s
            })
            .center(),
        )
        .child(
            TextView::new({
                let mut s = StyledString::new();

                s.append_plain("Press ");
                s.append_styled("esc", Color::Dark(BaseColor::Yellow));
                s.append_plain(" to close");

                s
            })
            .center(),
        );

    let panel = Panel::new(layout).title("Error");
    let panel = OnEventView::new(panel).on_event(Key::Esc, |s| {
        s.pop_layer();
    });

    siv.add_layer(panel);
}

fn previous_chapter(siv: &mut Cursive, state: &State) {
    siv.pop_layer();

    if state.chapter == 1 {
        load_url(siv, &state.url);
        reader_view(siv);
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

    let mut data = Data::load();

    if data.is_err() {
        error_panel(siv, "Data missing/corrupted. Regenerating");
        let res = Data::new().save();

        assert!(res.is_ok(), "Failed to generate data. Please check that {:?} is accessible by your user. Error was: {}",
                Data::data_folder().yellow(), res.unwrap_err().yellow());

        data = Data::load();
    }

    let data = data.unwrap();

    let mut main_view = LinearLayout::vertical();

    main_view.add_child(
        TextView::new({
            let mut s = StyledString::new();

            s.append_styled("Welcome to `lncli`", Effect::Bold);

            s
        })
        .center()
        .full_width(),
    );

    let mut tracked_count = StyledString::new();

    tracked_count.append_plain("There are currently ");
    tracked_count.append_styled(
        format!("{}", data.tracked().len()),
        Color::Dark(BaseColor::Yellow),
    );
    tracked_count.append_plain(" tracked novels.");

    main_view.add_child(TextView::new(tracked_count).center().full_width());
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

    let tv = create_tv(&data, updates, submit);

    let tracked_panel = Panel::new(tv).title("Tracked Novels").full_screen();

    let rv = create_rv(&data, submit);

    let recent_panel = Panel::new(rv).title("Recent Novels").full_screen();

    main_view.add_child(
        LinearLayout::horizontal()
            .child(tracked_panel)
            .child(recent_panel),
    );

    let controls_text = get_home_controls();

    main_view.add_child(TextView::new(controls_text).align(Align::bot_right()));

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

fn get_home_controls() -> StyledString {
    let mut text = StyledString::new();

    text.append_styled("q", Color::Dark(BaseColor::Yellow));
    text.append_plain("uit, ");
    text.append_styled("r", Color::Dark(BaseColor::Yellow));
    text.append_plain("eader, ");
    text.append_styled("t", Color::Dark(BaseColor::Yellow));
    text.append_plain("rack/untrack, ");
    text.append_styled("s", Color::Dark(BaseColor::Yellow));
    text.append_plain("earch, ");
    text.append_styled("u", Color::Dark(BaseColor::Yellow));
    text.append_plain("pdate check, ");
    text.append_styled("enter", Color::Dark(BaseColor::Yellow));
    text.append_plain(" to select, ");
    text.append_styled("arrow keys", Color::Dark(BaseColor::Yellow));
    text.append_plain(" to navigate");

    text
}

fn create_rv(data: &Data, submit: fn(&mut Cursive, &LN)) -> OnEventView<NamedView<SelectView<LN>>> {
    let label = |name: &str, last_chapter: usize| {
        let mut s = StyledString::new();

        s.append_styled(
            name,
            Style::merge(&[Color::Dark(BaseColor::Red).into(), Effect::Bold.into()]),
        );
        s.append_plain(" (");
        s.append_styled(
            format!("{}", last_chapter),
            Style::merge(&[Color::Dark(BaseColor::Magenta).into(), Effect::Bold.into()]),
        );
        s.append_plain(")");

        s
    };

    let rv = {
        let mut sv = SelectView::new();
        for x in data.recent().iter() {
            sv.add_item(label(&x.name, x.last_chapter), x.clone());
        }

        sv.set_on_submit(submit);

        sv.h_align(HAlign::Center).with_name("recent_view")
    };

    let data = data.clone();

    OnEventView::new(rv).on_event('t', move |s| {
        let mut data = data.clone();
        let rv = s.find_name::<SelectView<LN>>("recent_view");

        if rv.is_none() {
            return;
        }
        let rv = rv.unwrap();

        let selected = rv.selected_id();

        if selected.is_none() {
            return;
        }

        let selected = selected.unwrap();
        let item = rv.get_item(selected).unwrap();

        data.add_tracked(item.1.clone());

        if let Err(e) = data.save() {
            error_panel(s, &format!("Failed to save data: {}", e));
            error!("Failed to save data: {}", e);
        }

        home_view(s, &None);
    })
}

fn create_tv(
    data: &Data,
    updates: &Option<Vec<LN>>,
    submit: fn(&mut Cursive, &LN),
) -> OnEventView<NamedView<SelectView<LN>>> {
    let label = |name: &str, last_chapter: usize, update: bool| {
        let mut s = StyledString::new();

        if update {
            s.append_styled(
                "*",
                Style::merge(&[Color::Dark(BaseColor::Magenta).into(), Effect::Bold.into()]),
            );
            s.append_plain(" ");
        }

        s.append_styled(
            name,
            Style::merge(&[Color::Dark(BaseColor::Red).into(), Effect::Bold.into()]),
        );
        s.append_plain(" (");
        s.append_styled(
            format!("{}", last_chapter),
            Style::merge(&[Color::Dark(BaseColor::Magenta).into(), Effect::Bold.into()]),
        );
        s.append_plain(")");

        s
    };

    let updates = updates.clone();
    let tv = {
        let mut sv = SelectView::new();
        for x in data.tracked() {
            if let Some(updates) = &updates {
                if updates.iter().any(|u| u.name == x.name) {
                    sv.add_item(label(&x.name, x.last_chapter, true), x.clone());
                } else {
                    sv.add_item(label(&x.name, x.last_chapter, false), x.clone());
                }
            } else {
                sv.add_item(label(&x.name, x.last_chapter, false), x.clone());
            }
        }

        sv.set_on_submit(submit);

        sv.h_align(HAlign::Center).with_name("tracked_view")
    };

    let data = data.clone();

    OnEventView::new(tv).on_event('t', move |s| {
        let updates = &updates;
        let mut data = data.clone();
        let sv = s.find_name::<SelectView<LN>>("tracked_view");

        if sv.is_none() {
            return;
        }
        let sv = sv.unwrap();

        let selected = sv.selected_id();

        if selected.is_none() {
            return;
        }

        let selected = selected.unwrap();
        let item = sv.get_item(selected).unwrap();

        data.tracked_mut().retain(|x| x != item.1);

        if let Err(e) = data.save() {
            error_panel(s, &format!("Failed to save data: {}", e));
            error!("Failed to save data: {}", e);
        }

        home_view(s, updates);
    })
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

        check.map_or_else(
            |ch| {
                error_panel(cursive, &format!("Failed to check for updates: {}", ch));
            },
            |check| {
                if check {
                    found.push(x.clone());
                }
            },
        );
    }

    home_view(cursive, &Some(found));
}

fn load_url(siv: &mut Cursive, url: &str) {
    info!("LOAD_URL: Attempting to load: {}", url.green());
    let output = scrape::load(url);

    if let Err(e) = output {
        error_panel(siv, &e.to_string());
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
        url: {
            // ugly ass hack
            let mut a = url.split_once("/chapter").unwrap().0.to_string();
            a.push_str(".html");

            a
        },
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

            if text.len() < 3 {
                error_panel(s, "please enter at least 3 characters");
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

            layout.add_child(
                TextView::new({
                    let mut s = StyledString::new();

                    s.append_styled(query, Color::Dark(BaseColor::Green));

                    s
                })
                .h_align(HAlign::Center)
                .no_wrap(),
            );
            layout.add_child(
                TextView::new("-".repeat(query.len() + 2))
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
        let mut s = StyledString::new();

        if results_mode {
            s.append_styled("t", Color::Dark(BaseColor::Yellow));
            s.append_plain("rack, ");
            s.append_styled("esc", Color::Dark(BaseColor::Yellow));
            s.append_plain(" to go back, ");
            s.append_styled("enter", Color::Dark(BaseColor::Yellow));
            s.append_plain(" to select, ");
            s.append_styled("arrow keys", Color::Dark(BaseColor::Yellow));
            s.append_plain(" to navigate");
        } else {
            s.append_styled("esc", Color::Dark(BaseColor::Yellow));
            s.append_plain(" to go back, ");
            s.append_styled("enter", Color::Dark(BaseColor::Yellow));
            s.append_plain(" to search");
        }

        s
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
            data.add_tracked(LN {
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
        error_panel(siv, &e.to_string());
        return;
    }

    let output = output.unwrap();

    search_view(siv, Some(output));
}
