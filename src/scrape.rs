#![allow(unused)]

use color_eyre::eyre::{bail, eyre};
use log::info;
use owo_colors::OwoColorize;
use ureq::Agent;
use url::Url;
use visdom::Vis;

use crate::Res;

pub fn search(input: &str) -> Res<Search> {
    bail!("searching is not implemented yet lmfao, nerd")
}

#[derive(Debug)]
pub struct Search {
    pub results: Vec<(Url, String)>,
}

pub fn load(url: &str) -> Res<Output> {
    let agent = Agent::new();

    let (main_url, chapter) = {
        let split = url.split_once("/chapter-").unwrap();
        let chapter = split.1.split_once(".html").unwrap().0;
        let chapter = chapter.parse::<usize>()?;

        let mut main_url = split.0.to_owned();
        main_url.push_str(".html");

        (main_url, chapter)
    };
    info!("Found main url: {}", main_url.green());

    let max_chapters = {
        let res = agent.get(&main_url).call()?;

        let dom = Vis::load(res.into_string()?).map_err(|e| eyre!("{}", e.green()))?;

        let item = dom.find("body > div.main > div > div > div.col-content > div.m-newest1 > ul > li:nth-child(1) > a").text().to_owned();

        let max_chapters = item.split_once('-').unwrap().0;
        let max_chapters = max_chapters.split_once(' ').unwrap().1;

        max_chapters.trim().parse::<usize>()?
    };

    info!("Found chapter: {}", chapter.yellow());
    info!("Found max chapters: {}", max_chapters.yellow());

    let (name, chapter_title, content) = {
        let res = agent.get(url).call()?;

        if res.status() != 200 {
            bail!(
                "got status code {}: {}",
                res.status().yellow(),
                res.status_text().green()
            );
        }

        let html = res.into_string()?;
        let dom = Vis::load(html).map_err(|e| eyre!("{}", e.green()))?;

        let name = dom
            .find("#main1 > div > div > div.top > h1 > a")
            .text()
            .to_owned();
        info!("Found title: {}", name.green());
        let chapter_title = {
            let el = dom.find("#main1 > div > div > div.top > span");
            let chapter_title = el.text().to_owned();

            chapter_title.split_once('-').unwrap().1.trim().to_owned()
        };
        info!("Found chapter title: {}", chapter_title.green());

        let content = {
            let top = dom.find("#main1 > div > div > div.txt");

            let mut content = String::new();

            for x in top.children("p") {
                let text = x.text_content();

                if !text.trim().is_empty() {
                    content.push_str(&format!("{}\n\n", text));
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

#[derive(Debug, Clone)]
pub struct Output {
    pub name: String,
    pub chapter_title: String,
    pub content: String,
    pub chapter: usize,
    pub max_chapters: usize,
}
