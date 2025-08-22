use eframe::egui::{Color32, RichText, TextEdit, TextStyle, Ui};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    app::EditResult,
    app_data::DataIndex,
    app_display::{
        DESCRIPTION_ROWS, FIELD_HORIZONTAL_SPACE, FIELD_VERTICAL_SPACE, NOTES_ROWS, ShowEdit,
        ShowEditInfo, show_edit_frame, show_edit_item, show_edit_stringlist_italics,
    },
    localize::fl,
    managed_list::{DistrictRef, Named, StringList},
};

#[derive(Default, Clone, PartialEq)]
pub struct Person {
    name: String,
    summary: String,
    found_in: Option<DistrictRef>,
    description: String,
    personality: StringList, // just 3?
    notes: String,
    // connections?
    // faction?
    // home?
}

impl Person {
    pub fn set_found_in(&mut self, found_in: Option<DistrictRef>) {
        if self.found_in.is_some() {
            warn!("replacing found_in of {} when it is not empty", self.name);
        }
        self.found_in = found_in;
    }
}

impl Named for Person {
    fn name(&self) -> &str {
        &self.name
    }

    fn display_name(&self) -> String {
        format!(
            "{}{}",
            self.name,
            if !self.summary.is_empty() {
                format!(" ({})", self.summary)
            } else {
                String::new()
            },
        )
    }

    fn make_data_index(index: usize) -> DataIndex {
        DataIndex::PersonIndex(index)
    }

    fn fetch_data_index(index: DataIndex) -> Option<usize> {
        match index {
            DataIndex::PersonIndex(ind) => Some(ind),
            _ => None,
        }
    }

    fn display_fields(&self) -> Vec<String> {
        vec![self.name.clone()]
    }

    fn display_headings() -> Vec<RichText> {
        vec![RichText::new(fl!("name_heading"))]
    }
}

impl ShowEdit for Person {
    fn show_edit(&mut self, ui: &mut Ui, item_info: ShowEditInfo) -> Option<EditResult> {
        show_edit_frame(ui, fl!("main_item_person"), "person", item_info, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(fl!("name_heading")).small().weak());
                        ui.horizontal(|ui| {
                            ui.add(TextEdit::singleline(&mut self.name).font(TextStyle::Heading));
                            if item_info.name_collision() {
                                let no_text = RichText::new("X").color(Color32::RED).strong();
                                ui.label(no_text);
                            }
                        });
                    });

                    ui.add_space(FIELD_HORIZONTAL_SPACE);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(fl!("summary_heading")).small().weak());
                        ui.add(TextEdit::singleline(&mut self.summary));
                    });
                });

                ui.add_space(FIELD_VERTICAL_SPACE);
                ui.label(RichText::new(fl!("description_heading")).small().weak());
                ui.add(
                    TextEdit::multiline(&mut self.description)
                        .desired_width(ui.available_width())
                        .desired_rows(DESCRIPTION_ROWS),
                );

                ui.add_space(FIELD_VERTICAL_SPACE);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(fl!("found_in_heading")).small().weak());
                        show_edit_item(
                            "found_in",
                            &mut self.found_in,
                            item_info.app_data().district_list(),
                            ui,
                        );
                    });

                    ui.add_space(FIELD_HORIZONTAL_SPACE);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(fl!("personality_heading")).small().weak());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("(").italics());
                            show_edit_stringlist_italics("personality", &mut self.personality, ui);
                            ui.label(RichText::new(")").italics());
                        });
                    });
                });

                ui.add_space(FIELD_VERTICAL_SPACE * 2.0);
                ui.label(RichText::new(fl!("notes_heading")).small().weak());
                ui.add(
                    TextEdit::multiline(&mut self.notes)
                        .desired_width(ui.available_width())
                        .desired_rows(NOTES_ROWS),
                );
            });
        })
    }
}

// ---------------
// PersonStore version 2
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PersonStore2 {
    name: String,
    summary: String,
    pub found_in: Option<String>, // district name
    description: String,
    personality: Vec<String>,
    notes: String,
}

impl From<&Person> for PersonStore2 {
    fn from(from_person: &Person) -> Self {
        PersonStore2 {
            name: from_person.name.clone(),
            summary: from_person.summary.clone(),
            found_in: from_person.found_in.as_ref().and_then(|i| i.name()),
            description: from_person.description.clone(),
            personality: from_person.personality.list().to_vec(),
            notes: from_person.notes.clone(),
        }
    }
}

impl From<PersonStore1> for PersonStore2 {
    fn from(from_store: PersonStore1) -> Self {
        PersonStore2 {
            name: from_store.name,
            summary: from_store.summary,
            found_in: None,
            description: from_store.description,
            personality: from_store.personality,
            notes: from_store.notes,
        }
    }
}

impl From<&PersonStore2> for Person {
    fn from(from_store: &PersonStore2) -> Self {
        Person {
            name: from_store.name.clone(),
            summary: from_store.summary.clone(),
            found_in: None, // added after creation
            description: from_store.description.clone(),
            personality: StringList::from_list(from_store.personality.clone()),
            notes: from_store.notes.clone(),
        }
    }
}

// ---------------
// PersonStore version 1
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PersonStore1 {
    name: String,
    summary: String,
    description: String,
    personality: Vec<String>,
    notes: String,
}

impl From<&Person> for PersonStore1 {
    fn from(from_person: &Person) -> Self {
        PersonStore1 {
            name: from_person.name.clone(),
            summary: from_person.summary.clone(),
            description: from_person.description.clone(),
            personality: from_person.personality.list().to_vec(),
            notes: from_person.notes.clone(),
        }
    }
}

impl From<&PersonStore1> for Person {
    fn from(from_store: &PersonStore1) -> Self {
        Person {
            name: from_store.name.clone(),
            summary: from_store.summary.clone(),
            found_in: None,
            description: from_store.description.clone(),
            personality: StringList::from_list(from_store.personality.clone()),
            notes: from_store.notes.clone(),
        }
    }
}
