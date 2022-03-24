#![warn(clippy::pedantic)]

use std::string::ToString;

use color_eyre::{eyre::bail, Report};
use reqwest::get;
use scrape::Output;
use url::{ParseError, Url};

use crate::scrape::scrape_content;

pub type Res<T> = Result<T, Report>;

mod scrape;

#[derive(Debug)]
pub enum Loaded {
    Yes,
    No(String),
    Never,
}

#[derive(Debug)]
pub struct App {
    loaded: Loaded,

    title: Option<String>,
    raw_lines: Vec<String>,

    url: Option<Url>,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self {
            loaded: Loaded::Never,

            title: None,
            raw_lines: Vec::new(),

            url: None,
        }
    }

    /// # Errors
    /// When the URL cannot be parsed.
    pub fn with_url(mut self, url: &str) -> Res<Self> {
        let res: Result<_, ParseError> = url.parse();

        if let Err(e) = res {
            self.loaded = Loaded::No(e.to_string());
            bail!(e);
        }

        self.url = Some(res?);

        Ok(self)
    }

    /// # Errors
    /// - When URL is not set
    /// # Panics
    /// - never
    pub async fn load(&mut self) -> Res<()> {
        if self.url.is_none() {
            self.loaded = Loaded::No("URL is not set".to_string());
            bail!("URL is not set");
        }

        let lines = &mut self.raw_lines;

        let url = self.url.as_ref().unwrap();

        let response = get(url.as_str()).await?;

        let body = response.text().await?;
        let Output { title, content } = scrape_content(&body)?;

        lines.clear();

        lines.extend(content.lines().map(ToString::to_string));

        self.title = Some(title);
        self.loaded = Loaded::Yes;

        Ok(())
    }

    #[must_use]
    pub fn get_title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    #[must_use]
    pub fn get_lines(&self) -> &[String] {
        &self.raw_lines
    }

    #[must_use]
    pub fn get_url(&self) -> Option<&Url> {
        self.url.as_ref()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
