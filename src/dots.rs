use std::fmt::Display;

use eframe::egui::{ComboBox, Ui};
use log::error;
use serde::{Deserialize, Serialize};




#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dots {
    #[default]
    Zero = 0,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Dots {
    pub fn combo_list ( ) -> &'static [&'static str] {  // todo: is this needed?
        DOT_STRINGS
    }

    pub fn show_edit ( &mut self, name: &str, ui: &mut Ui ) {
        let mut selected = *self as usize;
        ComboBox::from_id_salt(name)
            .show_index(ui, &mut selected, DOT_STRINGS.len(), |i| DOT_STRINGS[i].to_string());
        *self = selected.into();
    }
}

impl From<Dots> for usize {
    fn from ( value: Dots ) -> Self {
        value as usize
    }
}

impl From<usize> for Dots {
    fn from ( value: usize ) -> Self {
        use Dots::*;

        match value {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            _ => { error!("converting {value} to Dots"); Zero },
        }
    }
}

const DOT_STRINGS: &[&str] = &[
    "○",
    "●",
    "●●",
    "●●●",
    "●●●●",
    "●●●●●",
];

impl Display for Dots {
    fn fmt ( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
        write!(f, "{}", DOT_STRINGS[*self as usize])
    }
}
