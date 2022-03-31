use color_eyre::eyre::bail;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::Res;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LN {
    pub name: String,
    pub url: String,
    pub last_chapter: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    tracked_novels: Vec<LN>,
    recent_novels: VecDeque<LN>,
}

impl Data {
    pub fn data_folder() -> PathBuf {
        let config_dir = dirs::config_dir().expect("Could not find config directory");

        config_dir.join("lncli/")
    }

    /// makes a new data
    pub fn new() -> Self {
        Self {
            tracked_novels: Vec::new(),
            recent_novels: VecDeque::new(),
        }
    }

    /// load data from file
    pub fn load() -> Res<Self> {
        let path = Self::data_folder().join("data.json");

        if !path.exists() {
            bail!(
                "data file does not exist: {}",
                path.display().to_string().green()
            );
        }

        let data = std::fs::read_to_string(&path)?;

        Ok(serde_json::from_str(&data)?)
    }

    /// save the data to the data file
    pub fn save(&self) -> Res<()> {
        let path = Self::data_folder().join("data.json");

        std::fs::create_dir_all(path.parent().unwrap())?;

        let mut data = self.clone();

        data.prune();

        let data = serde_json::to_string_pretty(&data)?;

        std::fs::write(&path, data)?;

        Ok(())
    }

    fn prune(&mut self) {
        if self.recent_novels.len() > 10 {
            self.recent_novels = self
                .recent_novels
                .clone()
                .into_iter()
                .fold(VecDeque::new(), |mut acc, x| {
                    if !acc.contains(&x) {
                        acc.push_back(x);
                    }
                    acc
                })
                .into_iter()
                .take(10)
                .collect::<VecDeque<_>>();
        }
    }

    /// get tracked novels
    pub fn tracked(&self) -> &Vec<LN> {
        &self.tracked_novels
    }

    /// get tracked novels but mutable
    pub fn tracked_mut(&mut self) -> &mut Vec<LN> {
        &mut self.tracked_novels
    }

    /// get recent novels
    pub fn recent(&self) -> &VecDeque<LN> {
        &self.recent_novels
    }

    /// get recent novels but mutable
    pub fn recent_mut(&mut self) -> &mut VecDeque<LN> {
        &mut self.recent_novels
    }
}
