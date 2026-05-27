// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::theme;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::core::model::Sort;

pub const SETTINGS_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
    pub app_theme: AppTheme,
    pub obs_url: Option<url::Url>,
    pub genius_token: Option<String>,
    pub song_sort: Option<Sort>,
    pub image_sort: Option<Sort>,
    pub video_sort: Option<Sort>,
    pub presentation_sort: Option<Sort>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            obs_url: None,
            genius_token: None,
            song_sort: None,
            image_sort: None,
            video_sort: None,
            presentation_sort: None,
        }
    }
}

#[derive(
    Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize, Default,
)]
pub struct PersistentState {
    pub recent_files: VecDeque<PathBuf>,
}
