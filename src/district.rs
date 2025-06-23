
use eframe::egui::{RichText, TextEdit, TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, localize::fl, managed_list::Named};



#[allow(dead_code)]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct District {
    name: String,
    // ? is there other data to store?
}

#[allow(dead_code)]
impl District {
    pub fn new ( name: &str ) -> Self {
        District {
            name: name.to_string(),
        }
    }
}

// ---------------------------
impl Named for District {
    fn name ( &self ) -> &str {
        &self.name
    }

    fn make_data_index ( index: usize ) -> DataIndex {
        DataIndex::DistrictIndex(index)
    }

    fn fetch_data_index ( index: DataIndex ) -> Option<usize> {
        match index {
            DataIndex::DistrictIndex( ind ) => Some(ind),
            _ => None,
        }
    }

    fn display_fields ( &self ) -> Vec<String> {
        vec![self.name.clone()]
    }

    fn display_headings ( ) -> Vec<RichText> {
        vec![RichText::new(fl!("name_heading"))]
    }
}

impl District {
    pub fn show_edit ( &mut self, ui: &mut Ui ) {
        ui.vertical(|ui| {
            // let name_heading = RichText::new(&self.name).heading();
            ui.add(TextEdit::singleline(&mut self.name).font(TextStyle::Heading));
            // ui.label(name_heading);
        });
    }
}
