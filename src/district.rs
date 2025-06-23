
use std::sync::WaitTimeoutResult;

use eframe::egui::{Color32, Label, RichText, Sense, TextEdit, TextStyle, Ui};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{app::EditResult, app_data::DataIndex, localize::fl, managed_list::Named};



#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
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
    pub fn show_edit ( &mut self, ui: &mut Ui, name_in_use: bool ) -> Option<EditResult> {
        const EDGE_SPACER: f32 = 6.0;
        const HEAD_SPACE: f32 = 6.0;
        let mut result = None;

        ui.horizontal(|ui| {
            if ui.add(Label::new(RichText::new("<").heading()).sense(Sense::click())).clicked() {
                debug!("return from edit district");
                result = Some(EditResult::Ignore);  // should this be return?
            }
            ui.add_space(EDGE_SPACER);

            ui.vertical(|ui| {
                ui.label(RichText::new("District").heading().strong().underline());
                ui.add_space(HEAD_SPACE);

                let name_text = RichText::new("Name").small().weak();
                ui.label(name_text);
                ui.horizontal(|ui| {
                    ui.add(TextEdit::singleline(&mut self.name).font(TextStyle::Heading));
                    if name_in_use {
                        let no_text = RichText::new("X").color(Color32::RED).strong();
                        ui.label(no_text);
                    }
                });
            });
        });

        result
    }
}
