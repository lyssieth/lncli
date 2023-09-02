#![allow(unused)]

use color_eyre::eyre::{bail, eyre};
use log::info;
use owo_colors::OwoColorize;
use regex::Regex;
use reqwest::blocking::Client;
use std::fmt::Write;
use url::Url;
use visdom::Vis;

use crate::Res;

const SEARCH_URL: &str = "https://freewebnovel.com/search/";

pub fn search(input: &str) -> Res<Search> {
    let client = Client::new();

    let res = client
        .post(SEARCH_URL)
        .form(&[("searchkey", input)])
        .send()?
        .error_for_status()?;

    let dom = Vis::load(res.text()?).map_err(|e| eyre!("{}", e.green()))?;

    let elements = dom.find("div.li-row");

    let mut results = Vec::new();

    for el in elements {
        let children = el.children();
        let title = children.find("h3.tit").first();
        let title = title.text();
        let url = children.find("h3.tit > a").first().attr("href");

        if url.is_none() {
            bail!("found no results"); // TODO: Nicer error messages
        }

        let Some(url) = url else {
            bail!("found no results"); // TODO: Nicer error messages
        };

        let url = format!("https://freewebnovel.com{}", url.to_string());

        results.push((
            Url::parse(&url).expect("Failed to parse URL"),
            title.clone(),
        ));
    }

    Ok(Search {
        query: input.to_owned(),
        results,
    })
}

#[derive(Debug)]
pub struct Search {
    pub query: String,
    pub results: Vec<(Url, String)>,
}

pub fn get_name(url: &str) -> Res<String> {
    let client = Client::new();

    let res = client.get(url).send()?.error_for_status()?;

    let dom = Vis::load(res.text()?).map_err(|e| eyre!("{}", e.green()))?;

    let title = dom.find("h1.tit").first();

    Ok(title.text())
}

pub fn load(url: &str) -> Res<Output> {
    if !url.contains("/chapter-") {
        bail!("invalid url: {}", url.green()); // TODO: Nicer error messages
    }

    let chapter_regex = Regex::new(r"chapter-(\d+)\.html").expect("Failed to create regex");

    let client = Client::new();

    let (main_url, chapter) = {
        let split = url
            .split_once("/chapter-")
            .expect("Failed to split URL at /chapter-");
        let chapter = split
            .1
            .split_once(".html")
            .expect("Failed to split chapter number")
            .0;
        let chapter = chapter.parse::<usize>()?;

        let mut main_url = split.0.to_owned();
        main_url.push_str(".html");

        (main_url, chapter)
    };
    info!("Found main url: {}", main_url.green());

    let max_chapters = {
        let res = client.get(&main_url).send()?.error_for_status()?;

        let dom = Vis::load(res.text()?).map_err(|e| eyre!("{}", e.green()))?;

        let item = dom.find("body > div.main > div > div > div.col-content > div.m-newest1 > ul > li:nth-child(1) > a").attr("href").expect("Failed to get href").to_string();

        let max_chapters = chapter_regex.captures(&item);

        let max_chapters = if let Some(max_chapters) = max_chapters {
            max_chapters
                .get(1)
                .expect("Failed to get max chapters")
                .as_str()
        } else {
            bail!("could not find max chapters, regex failed"); // TODO: Nicer error messages
        };

        max_chapters.trim().parse::<usize>()?
    };

    info!("Found chapter: {}", chapter.yellow());
    info!("Found max chapters: {}", max_chapters.yellow());

    let (name, chapter_title, content) = {
        let res = client.get(url).send()?.error_for_status()?;

        let html = res.text()?;
        let dom = Vis::load(html).map_err(|e| eyre!("{}", e.green()))?;

        let name = dom.find("#main1 > div > div > div.top > h1 > a").text();
        info!("Found title: {}", name.green());
        let chapter_title = {
            let el = dom.find("#main1 > div > div > div.top > span");
            let chapter_title = el.text();

            chapter_title
                .clone()
                .split_once('-')
                .map_or(chapter_title, |split| split.1.trim().to_owned())
        };
        info!("Found chapter title: {}", chapter_title.green());

        let content = {
            let top = dom.find("#main1 > div > div > div.txt");

            let mut content = String::new();

            for x in top.children("p") {
                let text = x.text_content();

                if !text.trim().is_empty() {
                    write!(&mut content, "{text}\n\n");
                }
            }

            content
        };

        (name, chapter_title, content)
    };

    Ok(Output {
        name,
        chapter_title,
        content,
        chapter,
        max_chapters,
    })
}

pub fn update_check(url: &str, last_chapter: usize) -> Res<bool> {
    if url.contains("/chapter-") {
        bail!("invalid url: {}", url.green()); // TODO: Nicer error messages
    }

    let chapter_regex = Regex::new(r"chapter-(\d+)\.html").expect("Failed to create regex");

    let client = Client::new();

    let res = client.get(url).send()?.error_for_status()?;

    let html = res.text()?;

    let dom = Vis::load(html).map_err(|e| eyre!("{}", e.green()))?;

    let max_chapters = {
        let item = dom.find("body > div.main > div > div > div.col-content > div.m-newest1 > ul > li:nth-child(1) > a").attr("href").expect("Failed to get href attribute").to_string();

        let max_chapters = chapter_regex.captures(&item);

        let max_chapters = if let Some(max_chapters) = max_chapters {
            max_chapters
                .get(1)
                .expect("Failed to get max chapters")
                .as_str()
        } else {
            bail!("could not find max chapters, regex failed"); // TODO: Nicer error messages
        };

        max_chapters.trim().parse::<usize>()?
    };

    Ok(max_chapters > last_chapter)
}

#[derive(Debug, Clone)]
pub struct Output {
    pub name: String,
    pub chapter_title: String,
    pub content: String,
    pub chapter: usize,
    pub max_chapters: usize,
}
