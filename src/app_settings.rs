use std::path::Path;

use anyhow::{Ok, Result, anyhow};
use log::info;
use serde::{Deserialize, Serialize};

use crate::app::{load_from_pot, save_to_pot};

const SETTINGS_NAME: &str = "settings";
const SETTINGS_EXTENSION: &str = "pot";

// TODO: Should include settings panel

#[derive(Debug, Clone)]
pub struct AppSettings {
    theme: eframe::egui::Theme,
}

// todo: load on start

#[allow(dead_code)]
impl AppSettings {
    pub fn theme(&self) -> eframe::egui::Theme {
        self.theme
    }

    pub fn save_to_file(&self, config_path: &Path) -> anyhow::Result<()> {
        let file_path = config_path
            .with_file_name(SETTINGS_NAME)
            .with_extension(SETTINGS_EXTENSION);
        let settings: SaveSettings1 = self.into();
        info!("Saving settings to file in {}", config_path.display());
        save_to_pot(&file_path, &settings)
    }

    // Create settings by loading from the settings file in the passed config directory
    pub fn load_from_file(config_path: &Path) -> Result<AppSettings> {
        let file_path = config_path
            .with_file_name(SETTINGS_NAME)
            .with_extension(SETTINGS_EXTENSION);
        info!("Attempting to load settings from {}", file_path.display());
        load_settings(&file_path)
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: eframe::egui::Theme::Light,
        }
    }
}

fn load_settings(file_path: &Path) -> Result<AppSettings> {
    // load save settings 1
    let data = load_from_pot::<SaveSettings1>(file_path)?;
    if data.validate() {
        // convert to AppSettings
        let ret = data.into();
        Ok(ret)
    } else {
        Err(anyhow!("unable to validate saved settings"))
    }
}

// ====================
// SaveSettings1
const SAVE1_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct SaveSettings1 {
    save_version: u16,
    theme: Theme,
}

impl SaveSettings1 {
    fn validate(&self) -> bool {
        self.save_version == SAVE1_VERSION
    }
}

impl From<SaveSettings1> for AppSettings {
    fn from(value: SaveSettings1) -> Self {
        AppSettings {
            theme: value.theme.into(),
        }
    }
}

impl From<&AppSettings> for SaveSettings1 {
    fn from(value: &AppSettings) -> Self {
        SaveSettings1 {
            save_version: SAVE1_VERSION,
            theme: value.theme.into(),
        }
    }
}

// -----------------------------
// eGUI Theme Save

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
enum Theme {
    Dark,
    #[default]
    Light,
}

impl From<eframe::egui::Theme> for Theme {
    fn from(value: eframe::egui::Theme) -> Self {
        match value {
            eframe::egui::Theme::Dark => Theme::Dark,
            eframe::egui::Theme::Light => Theme::Light,
        }
    }
}

impl From<Theme> for eframe::egui::Theme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Dark => eframe::egui::Theme::Dark,
            Theme::Light => eframe::egui::Theme::Light,
        }
    }
}
