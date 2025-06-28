
use eframe::egui::{Color32, Label, Margin, RichText, Sense, Stroke, TextEdit, TextStyle, Ui};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{app::EditResult, app_data::DataIndex, app_display::{ShowEdit, ShowEditInfo}, dots::Dots, localize::fl, managed_list::Named};



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
        const EDGE_SPACER: f32 = 6.0;
        const HEAD_SPACE: f32 = 6.0;
        const STROKE_WIDTH: f32 = 1.;
        const STROKE_COLOR: Color32 = Color32::GRAY;
        const INNER_MARGIN: Margin = Margin::same(6);
        const FIELD_VERTICAL_SPACE: f32 = 10.0;
        const FIELD_HORIZONTAL_SPACE: f32 = 20.0;

        let mut result = None;

        ui.horizontal(|ui| {
            if ui.add(Label::new(RichText::new("<").heading()).sense(Sense::click())).clicked() {
                debug!("return from edit district");
                result = Some(EditResult::Ignore);  // should this be return?
            }
            ui.add_space(EDGE_SPACER);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(fl!("main_item_district")).heading().strong().underline());
                    if item_info.show_save() {
                        ui.add_space(60.0);
                        if ui.button(fl!("edit_save")).clicked() {
                            debug!("save edited district requested");
                            result = Some(EditResult::Submit);
                        }
                    }
                });
                ui.add_space(HEAD_SPACE);

                eframe::egui::Frame::default()
                    .stroke(Stroke::new(STROKE_WIDTH, STROKE_COLOR))
                    .inner_margin(INNER_MARGIN)
                    .show(ui, |ui| {
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
                            ui.add(TextEdit::multiline(&mut self.description));

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("wealth_heading")).small().weak());
                                    ui.add(TextEdit::singleline(&mut self.wealth.to_string()).font(TextStyle::Heading));

                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("safety_heading")).small().weak());
                                    ui.add(TextEdit::singleline(&mut self.safety.to_string()).font(TextStyle::Heading));

                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("crime_heading")).small().weak());
                                    ui.add(TextEdit::singleline(&mut self.crime.to_string()).font(TextStyle::Heading));

                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("occult_heading")).small().weak());
                                    ui.add(TextEdit::singleline(&mut self.occult.to_string()).font(TextStyle::Heading));

                                });
                            });

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.label(RichText::new(fl!("notes_heading")).small().weak());
                            ui.add(TextEdit::multiline(&mut self.description));

                        });
                    });
            });
        });

        result
    }
}
