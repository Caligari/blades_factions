
use eframe::egui::{Color32, RichText, TextEdit, TextStyle, Ui};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{app::EditResult, app_data::DataIndex, app_display::{show_edit_district, show_edit_districts, show_edit_frame, ShowEdit, ShowEditInfo, DESCRIPTION_ROWS, FIELD_HORIZONTAL_SPACE, FIELD_VERTICAL_SPACE, NOTES_ROWS}, clock::Clock, localize::fl, managed_list::{DistrictRef, DistrictRefList, FactionRef, Named, PersonRef}, tier::Tier};

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq)]
pub struct Faction {
    name: String,
    description: String,
    tier: Tier,
    hq: Option<DistrictRef>,
    turf: DistrictRefList, // Vec<DistrictRef>,
    leader: Option<PersonRef>,
    notable: Vec<PersonRef>,
    assets: String,
    notes: String,
    allies: Vec<FactionRef>,
    enemies: Vec<FactionRef>,
    general: String,
    clocks: Vec<Clock>,
}

impl Named for Faction {
    fn name ( &self ) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    fn make_data_index ( index: usize ) -> DataIndex {
        DataIndex::FactionIndex(index)
    }

    fn fetch_data_index ( index: DataIndex ) -> Option<usize> {
        match index {
            DataIndex::FactionIndex( ind ) => Some(ind),
            _ => None,
        }
    }

    fn display_fields ( &self ) -> Vec<String> {
        vec![
            self.name.clone(),
            self.tier.to_string(),
            self.hq.clone().map_or(String::new(), |d| d.name().map_or(String::new(), |s| s)),
            self.turf.list().iter()
                .map(|d| if let Some(d_name) = d.name() { d_name } else { error!("reference has no name when making turf list for display fields"); String::new() })
                .collect::<Vec<String>>()
                .join(", "),
            //hq loc?
        ]
    }

    fn display_headings ( ) -> Vec<RichText> {
        vec![
            RichText::new(fl!("name_heading")),
            RichText::new(fl!("tier_heading")),
            RichText::new(fl!("hq_heading")),
            RichText::new(fl!("turf_heading")),
        ]
    }
}

impl Faction {
    pub fn set_hq ( &mut self, hq: Option<DistrictRef> ) {
        if self.hq.is_some() { warn!("replacing hq of {} when it is not empty", self.name); }
        self.hq = hq;
    }

    pub fn set_leader ( &mut self, leader: Option<PersonRef> ) {
        if self.leader.is_some() { warn!("replacing leader of {} when it is not empty", self.name); }
        self.leader = leader;
    }

    pub fn set_turf ( &mut self, turf: Vec<DistrictRef> ) {
        if !self.turf.list().is_empty() { warn!("replacing turf of {} when it is not empty", self.name); }
        self.turf = DistrictRefList::from_list(turf);
    }

    pub fn set_notable ( &mut self, notable: Vec<PersonRef> ) {
        if !self.notable.is_empty() { warn!("replacing notable of {} when it is not empty", self.name); }
        self.notable = notable;
    }

    pub fn set_allies ( &mut self, allies: Vec<FactionRef> ) {
        if !self.allies.is_empty() { warn!("replacing allies of {} when it is not empty", self.name); }
        self.allies = allies;
    }

    pub fn set_enemies ( &mut self, enemies: Vec<FactionRef> ) {
        if !self.enemies.is_empty() { warn!("replacing enemies of {} when it is not empty", self.name); }
        self.enemies = enemies;
    }
}

impl ShowEdit for Faction {
    fn show_edit ( &mut self, ui: &mut Ui, item_info: ShowEditInfo ) -> Option<EditResult> {
        show_edit_frame(
            ui,
            fl!("main_item_faction"),
            "faction",
            item_info,
            |ui| {
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
                                    ui.label(RichText::new(fl!("tier_heading")).small().weak());
                                    self.tier.show_edit("tier", ui);
                                });
                            });

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.label(RichText::new(fl!("description_heading")).small().weak());
                            ui.add(TextEdit::multiline(&mut self.description)
                                .desired_width(ui.available_width())
                                .desired_rows(DESCRIPTION_ROWS)
                            );

                            ui.add_space(FIELD_VERTICAL_SPACE);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("hq_heading")).small().weak());
                                    show_edit_district("hq", &mut self.hq, item_info.app_data(), ui);
                                });

                                ui.add_space(FIELD_HORIZONTAL_SPACE);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(fl!("turf_heading")).small().weak());
                                    show_edit_districts("turf", &mut self.turf, item_info.app_data(), ui);
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



// -----------------------------
// Stored

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionStore {
    name: String,
    description: String,
    tier: Tier,
    pub hq: Option<String>,  // district name
    pub turf: Vec<String>,  // districts
    pub leader: Option<String>,  // person name
    pub notable: Vec<String>,  // people
    assets: String,
    notes: String,
    pub allies: Vec<String>,  // fations
    pub enemies: Vec<String>,  // factions
    general: String,
    clocks: Vec<Clock>,
}

impl From<&Faction> for FactionStore {
    fn from(from_faction: &Faction) -> Self {
        FactionStore {
            name: from_faction.name.clone(),
            description: from_faction.description.clone(),
            tier: from_faction.tier,
            hq: from_faction.hq.as_ref().and_then(|i| { i.name() }),
            turf: from_faction.turf.list().iter().filter_map(|i| { i.name() }).collect(),
            leader: from_faction.leader.as_ref().and_then(|i| { i.name() }),
            notable: from_faction.notable.iter().filter_map(|i| { i.name() }).collect(),
            assets: from_faction.assets.clone(),
            notes: from_faction.notes.clone(),
            allies: from_faction.allies.iter().filter_map(|i| { i.name() }).collect(),
            enemies: from_faction.enemies.iter().filter_map(|i| { i.name() }).collect(),
            general: from_faction.general.clone(),
            clocks: from_faction.clocks.clone(),
        }
    }
}

// todo from factionstore to faction, for loading
impl From<&FactionStore> for Faction {
    fn from(from_store: &FactionStore) -> Self {
        Faction {
            name: from_store.name.clone(),
            description: from_store.description.clone(),
            tier: from_store.tier,
            hq: None,  // added after creation
            turf: DistrictRefList::default(),  // added after creation
            leader: None,  // added after creation
            notable: Vec::new(),  // added after creation
            assets: from_store.assets.clone(),
            notes: from_store.notes.clone(),
            allies: Vec::new(),  // added after creation
            enemies: Vec::new(),  // added after creation
            general: from_store.general.clone(),
            clocks: from_store.clocks.clone(),
        }
    }
}
