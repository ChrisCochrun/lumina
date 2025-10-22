// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{
        self, CosmicConfigEntry,
        cosmic_config_derive::CosmicConfigEntry,
    },
    theme,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::PathBuf};

pub const SETTINGS_VERSION: u64 = 1;

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize,
)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(
    Clone,
    CosmicConfigEntry,
    Debug,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(default)]
pub struct Settings {
    pub app_theme: AppTheme,
    pub obs_url: Option<url::Url>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            obs_url: None,
        }
    }
}

#[derive(
    Clone,
    CosmicConfigEntry,
    Debug,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
)]
pub struct PersistentState {
    pub recent_files: VecDeque<PathBuf>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            recent_files: VecDeque::new(),
        }
    }
}
