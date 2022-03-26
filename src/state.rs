use crate::{scrape::Output, Res};

#[derive(Debug, Clone)]
pub struct State {
    pub url: String,
    pub title: String,
    pub chapter: usize,
    pub max_chapters: usize,
    pub content: String,
}

impl State {
    pub fn from_output(url: &str, output: Output) -> Res<Self> {
        let Output {
            chapter_title,
            content,
            chapter,
            max_chapters,
            ..
        } = output;
        Ok(Self {
            url: url.to_owned(),
            title: chapter_title,
            chapter,
            max_chapters,
            content,
        })
    }
}
