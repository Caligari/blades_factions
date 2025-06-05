use std::{collections::BTreeMap, sync::Arc};

use eframe::egui::mutex::RwLock;
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, clock::Clock, district::DistrictRef, person::PersonRef, tier::Tier};

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct Faction {
    name: String,
    description: String,
    tier: Tier,
    hq: Option<DistrictRef>,
    turf: Vec<DistrictRef>,
    leader: Option<PersonRef>,
    notable: Vec<PersonRef>,
    assets: String,
    notes: String,
    allies: Vec<FactionRef>,
    enemies: Vec<FactionRef>,
    general: String,
    clocks: Vec<Clock>,
}

#[allow(dead_code)]
pub type FactionRef = Arc<RwLock<FactionIndex>>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FactionIndex {
    name: String,
    index: usize,
}

#[allow(dead_code)]
impl FactionIndex {
    pub fn index ( &self ) -> DataIndex {
        DataIndex::FactionIndex(self.index)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct FactionList {
    factions: Vec<Faction>,
    factions_index: BTreeMap<String, FactionRef>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionStore {
    name: String,
    description: String,
    tier: Tier,
    hq: DataIndex,  // district index
    turf: Vec<DataIndex>,  // districts
    leader: DataIndex,  // person index
    notable: Vec<DataIndex>,  // people
    assets: String,
    notes: String,
    allies: Vec<DataIndex>,  // fations
    enemies: Vec<DataIndex>,  // factions
    general: String,
    clocks: Vec<Clock>,
}

impl From<&Faction> for FactionStore {
    fn from(from_faction: &Faction) -> Self {
        FactionStore {
            name: from_faction.name.clone(),
            description: from_faction.description.clone(),
            tier: from_faction.tier,
            hq: from_faction.hq.as_ref().map_or(DataIndex::Nothing, |i| {
                let index = i.read();
                index.index()
            }),
            turf: from_faction.turf.iter().map(|i| {
                let index = i.read();
                index.index()
            }).collect(),
            leader: from_faction.leader.as_ref().map_or(DataIndex::Nothing, |i| {
                let index = i.read();
                index.index()
            }),
            notable: from_faction.notable.iter().map(|i| {
                let index = i.read();
                index.index()
            }).collect(),
            assets: from_faction.assets.clone(),
            notes: from_faction.notes.clone(),
            allies: from_faction.allies.iter().map(|i| {
                let index = i.read();
                index.index()
            }).collect(),
            enemies: from_faction.enemies.iter().map(|i| {
                let index = i.read();
                index.index()
            }).collect(),
            general: from_faction.general.clone(),
            clocks: from_faction.clocks.clone(),
        }
    }
}
