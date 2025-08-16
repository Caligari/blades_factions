use eframe::egui::{Color32, RichText, TextEdit, TextStyle, Ui};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    app::EditResult,
    app_data::DataIndex,
    app_display::{
        DESCRIPTION_ROWS, FIELD_HORIZONTAL_SPACE, FIELD_VERTICAL_SPACE, NOTES_ROWS, ShowEdit,
        ShowEditInfo, show_edit_frame, show_edit_list,
    },
    dots::Dots,
    localize::fl,
    managed_list::{Named, PersonRef, PersonRefList},
};

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq)]
pub struct District {
    name: String,
    description: String,
    wealth: Dots,
    safety: Dots,
    crime: Dots,
    occult: Dots,
    notable: PersonRefList,
    notes: String,
}

#[allow(dead_code)]
impl District {
    pub fn new(name: &str) -> Self {
        District {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn set_notable(&mut self, notable: Vec<PersonRef>) {
        if !self.notable.list().is_empty() {
            warn!("replacing notable of {} when it is not empty", self.name);
        }
        self.notable = PersonRefList::from_list(notable);
    }
}

// ---------------------------
impl Named for District {
    fn name(&self) -> &str {
        &self.name
    }

    fn display_name(&self) -> String {
        self.name.clone()
    }

    fn make_data_index(index: usize) -> DataIndex {
        DataIndex::DistrictIndex(index)
    }

    fn fetch_data_index(index: DataIndex) -> Option<usize> {
        match index {
            DataIndex::DistrictIndex(ind) => Some(ind),
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

impl ShowEdit for District {
    fn show_edit(&mut self, ui: &mut Ui, item_info: ShowEditInfo) -> Option<EditResult> {
        show_edit_frame(ui, fl!("main_item_district"), "district", item_info, |ui| {
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
                ui.add(
                    TextEdit::multiline(&mut self.description)
                        .desired_width(ui.available_width())
                        .desired_rows(DESCRIPTION_ROWS),
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
                ui.vertical(|ui| {
                    ui.label(RichText::new(fl!("notables_heading")).small().weak());
                    show_edit_list(
                        "notables",
                        &mut self.notable,
                        item_info.app_data().person_list(),
                        ui,
                    );
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

#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DistrictStore {
    name: String,
    description: String,
    wealth: Dots,
    safety: Dots,
    crime: Dots,
    occult: Dots,
    pub notable: Vec<String>, // people
    notes: String,
}

impl From<&District> for DistrictStore {
    fn from(from_district: &District) -> Self {
        DistrictStore {
            name: from_district.name.clone(),
            description: from_district.description.clone(),
            notable: from_district
                .notable
                .list()
                .iter()
                .filter_map(|i| i.name())
                .collect(),
            notes: from_district.notes.clone(),
            wealth: from_district.wealth,
            safety: from_district.safety,
            crime: from_district.crime,
            occult: from_district.occult,
        }
    }
}

impl From<&DistrictStore> for District {
    fn from(from_store: &DistrictStore) -> Self {
        District {
            name: from_store.name.clone(),
            description: from_store.description.clone(),
            notable: PersonRefList::default(), // added after creation
            notes: from_store.notes.clone(),
            wealth: Dots::default(),
            safety: Dots::default(),
            crime: Dots::default(),
            occult: Dots::default(),
        }
    }
}
