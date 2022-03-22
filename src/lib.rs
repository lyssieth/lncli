#![warn(clippy::pedantic)]

use color_eyre::Report;
use url::Url;

pub type Res<T> = Result<T, Report>;

pub struct App {
    raw_lines: Vec<String>,
    doc: Vec<String>,

    max_x: isize,
    max_y: isize,

    current_bottom: isize,
    current_top: isize,

    url: Option<Url>,
    path: String,
}

impl App {
    pub async fn new() -> Res<Self> {
        let mut s = Self {
            raw_lines: Vec::new(),
            doc: Vec::new(),

            max_x: -1,
            max_y: -1,

            current_bottom: -1,
            current_top: -1,

            url: None,
            path: String::new(),
        };

        Ok(s)
    }

    pub fn with_url(mut self, url: &str) -> Res<Self> {
        self.url = Some(url.parse()?);
        Ok(self)
    }
}
