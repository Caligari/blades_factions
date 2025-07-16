
use eframe::egui::{Color32, RichText, TextEdit, TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::{app::EditResult, app_data::DataIndex, app_display::{show_edit_frame, show_edit_stringlist_italics, ShowEdit, ShowEditInfo, DESCRIPTION_ROWS, FIELD_VERTICAL_SPACE, NOTES_ROWS}, localize::fl, managed_list::{Named, StringList}};





#[derive(Default, Clone, PartialEq)]
pub struct Person {
    name: String,
    description: String,
    personality: StringList, // just 3?
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

impl ShowEdit for Person {
    fn show_edit ( &mut self, ui: &mut Ui, item_info: ShowEditInfo ) -> Option<EditResult> {
        show_edit_frame(
            ui,
            fl!("main_item_person"),
            "person",
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

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.label(RichText::new(fl!("personality_heading")).small().weak());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("(").italics());
                                show_edit_stringlist_italics("personality", &mut self.personality, ui);
                                ui.label(RichText::new(")").italics());
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


#[allow(dead_code)]
#[derive(Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PersonStore {
    name: String,
    description: String,
    personality: Vec<String>,
    notes: String,
}

impl From<&Person> for PersonStore {
    fn from(from_person: &Person) -> Self {
        PersonStore {
            name: from_person.name.clone(),
            description: from_person.description.clone(),
            personality: from_person.personality.list().to_vec(),
            notes: from_person.notes.clone(),
        }
    }
}

impl From<&PersonStore> for Person {
    fn from(from_store: &PersonStore) -> Self {
        Person {
            name: from_store.name.clone(),
            description: from_store.description.clone(),
            personality: StringList::from_list(from_store.personality.clone()),
            notes: from_store.notes.clone(),
        }
    }
}
