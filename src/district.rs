
use eframe::egui::{Color32, RichText, TextEdit, TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::{app::EditResult, app_data::DataIndex, app_display::{show_edit_frame, ShowEdit, ShowEditInfo, DESCRIPTION_ROWS, FIELD_HORIZONTAL_SPACE, FIELD_VERTICAL_SPACE, NOTES_ROWS}, dots::Dots, localize::fl, managed_list::Named};



#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct District {
    name: String,
    description: String,
    wealth: Dots,
    safety: Dots,
    crime: Dots,
    occult: Dots,
    notes: String,
}

#[allow(dead_code)]
impl District {
    pub fn new ( name: &str ) -> Self {
        District {
            name: name.to_string(),
            ..Default::default()
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

impl ShowEdit for District {
    fn show_edit ( &mut self, ui: &mut Ui, item_info: ShowEditInfo ) -> Option<EditResult> {
        show_edit_frame(
            ui,
            fl!("main_item_district"),
            "district",
            item_info,
            |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(fl!("name_heading")).small().weak());
                            ui.horizontal(|ui| {
                                ui.add(TextEdit::singleline(&mut self.name).font(TextStyle::Heading));
                                if item_info.name_collision() {
                                    let no_text = RichText::new("X").color(Color32::RED).strong();
                                    ui.label(no_text);
                                }
                            });

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.label(RichText::new(fl!("description_heading")).small().weak());
                            ui.add(TextEdit::multiline(&mut self.description)
                                .desired_width(ui.available_width())
                                .desired_rows(DESCRIPTION_ROWS)
                            );

                            ui.add_space(FIELD_VERTICAL_SPACE * 2.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("wealth_heading")).small().weak());
                                    self.wealth.show_edit("wealth", ui);
                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("safety_heading")).small().weak());
                                    self.safety.show_edit("safety", ui);
                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("crime_heading")).small().weak());
                                    self.crime.show_edit("crime", ui);
                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("occult_heading")).small().weak());
                                    self.occult.show_edit("occult", ui);
                                });
                            });

                            ui.add_space(FIELD_VERTICAL_SPACE * 2.0);
                            ui.label(RichText::new(fl!("notes_heading")).small().weak());
                            ui.add(TextEdit::multiline(&mut self.notes)
                                .desired_width(ui.available_width())
                                .desired_rows(NOTES_ROWS)
                            );

                        });
                    }
        )
    }
}
