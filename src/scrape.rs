#![allow(unused)]

use color_eyre::eyre::bail;
use url::Url;

use crate::Res;

pub fn search(input: &str) -> Res<Search> {
    bail!("searching is not implemented yet lmfao, nerd")
}

#[derive(Debug)]
pub struct Search {
    pub results: Vec<(Url, String)>,
}

pub fn load(url: &str) -> Res<Output> {
    bail!("loading is not implemented yet lmfao, nerd");
}

#[derive(Debug)]
pub struct Output {
    pub title: String,
    pub content: String,
    pub chapter: usize,
    pub max_chapters: usize,
}
