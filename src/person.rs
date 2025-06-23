
use eframe::egui::{RichText, TextEdit, TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, localize::fl, managed_list::Named};





#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Person {
    name: String,
    description: String,
    // characteristics strings
    notes: String,
    // connections?
    // faction?
    // home?
}

impl Named for Person {
    fn name ( &self ) -> &str {
        &self.name
    }

    fn make_data_index ( index: usize ) -> DataIndex {
        DataIndex::PersonIndex(index)
    }

    fn fetch_data_index ( index: DataIndex ) -> Option<usize> {
        match index {
            DataIndex::PersonIndex( ind ) => Some(ind),
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

impl Person {
    pub fn show_edit ( &mut self, ui: &mut Ui ) {
        ui.vertical(|ui| {
            // let name_heading = RichText::new(&self.name).heading();
            ui.add(TextEdit::singleline(&mut self.name).font(TextStyle::Heading));
            // ui.label(name_heading);
        });
    }
}
