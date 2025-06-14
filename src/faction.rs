
use eframe::epaint::Vertex;
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, clock::Clock, localize::fl, managed_list::{DistrictRef, FactionRef, Named, PersonRef}, tier::Tier};

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
            //hq loc?
        ]
    }

    fn display_headings ( ) -> Vec<String> {
        vec![fl!("name_heading"), fl!("tier_heading")]
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
    hq: Option<String>,  // district name
    turf: Vec<String>,  // districts
    leader: Option<String>,  // person name
    notable: Vec<String>,  // people
    assets: String,
    notes: String,
    allies: Vec<String>,  // fations
    enemies: Vec<String>,  // factions
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
            turf: from_faction.turf.iter().filter_map(|i| { i.name() }).collect(),
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
impl From<FactionStore> for Faction {
    fn from(from_store: FactionStore) -> Self {
        Faction {
            name: from_store.name,
            description: from_store.description,
            tier: from_store.tier,
            hq: None,  // todo
            turf: Vec::new(),  // todo
            leader: None,  // todo
            notable: Vec::new(),  // todo
            assets: from_store.assets,  // todo
            notes: from_store.notes,
            allies: Vec::new(),  // todo
            enemies: Vec::new(),  // todo
            general: from_store.general,
            clocks: Vec::new(),  // todo
        }
    }
}
