#![allow(unused)]

use url::Url;

use crate::Res;

pub fn search(input: &str) -> Res<Search> {
    todo!("implement searching")
}

#[derive(Debug)]
pub struct Search {
    pub results: Vec<(Url, String)>,
}

pub fn load(url: &str) -> Res<Output> {
    todo!("implement loading")
}

#[derive(Debug)]
pub struct Output {
    pub title: String,
    pub content: String,
}
