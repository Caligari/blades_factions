use std::fmt::Display;

use eframe::egui::{ComboBox, FontFamily, FontId, Ui};
use log::error;
use serde::{Deserialize, Serialize};





#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Tier0 = 0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
}

impl Tier {
    pub fn show_edit ( &mut self, name: &str, ui: &mut Ui ) {
        let mut selected = *self as usize;
        ui.scope(|ui| {
            ui.style_mut().text_styles.insert(eframe::egui::TextStyle::Button, FontId::new(28.0, FontFamily::Proportional));
            ComboBox::from_id_salt(name)
                .show_index(ui, &mut selected, TIER_STRINGS.len(), |i| TIER_STRINGS[i].to_string());

        });

        *self = selected.into();
    }
}

impl From<Tier> for usize {
    fn from ( value: Tier ) -> Self {
        value as usize
    }
}

impl From<usize> for Tier {
    fn from ( value: usize ) -> Self {
        use Tier::*;

        match value {
            0 => Tier0,
            1 => Tier1,
            2 => Tier2,
            3 => Tier3,
            4 => Tier4,
            5 => Tier5,
            _ => { error!("converting {value} to Tier"); Tier0 },
        }
    }
}

const TIER_STRINGS: &[&str] = &[
    "0",
    "I",
    "II",
    "III",
    "IV",
    "V",
];


impl Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", TIER_STRINGS[*self as usize])
    }
}