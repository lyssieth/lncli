use crate::{
    scrape::{self, Output},
    Res,
};

#[derive(Debug, Clone)]
pub struct State {
    pub url: String,
    pub title: String,
    pub chapter: usize,
    pub max_chapters: usize,
    pub content: String,
}

impl State {
    pub fn from_url(url: &str) -> Res<Self> {
        let Output {
            title,
            content,
            chapter,
            max_chapters,
        } = scrape::load(url)?;
        Ok(Self {
            url: url.to_owned(),
            title,
            chapter,
            max_chapters,
            content,
        })
    }
}
