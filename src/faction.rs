use std::{collections::BTreeMap, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{district::DistrictIndex, person::PersonIndex, tier::Tier};


// todo: this needs a serializeable form, where we process the Indexes into index numbers
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Faction {
    name: String,
    description: String,
    tier: Tier,
    hq: Option<Rc<DistrictIndex>>,
    // turf districts
    leader: Option<Rc<PersonIndex>>,
    // notable persons
    assets: String,
    notes: String,
    // allies factions
    // enemies factions
    general: String,
    // clocks
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FactionIndex {
    name: String,
    index: usize,
}

#[allow(dead_code)]
impl FactionIndex {
    pub fn index ( &self ) -> usize {
        self.index
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FactionList {
    factions: Vec<Faction>,
    factions_index: BTreeMap<String, Rc<FactionIndex>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionStore {
    name: String,
    description: String,
    tier: Tier,
    hq: Option<usize>,  // district index
    // turf districts
    leader: Option<usize>,  // person index
    // notable persons
    assets: String,
    notes: String,
    // allies factions
    // enemies factions
    general: String,
    // clocks
}

impl From<Faction> for FactionStore {
    fn from(value: Faction) -> Self {
        FactionStore {
            name: value.name,
            description: value.description,
            tier: value.tier,
            hq: value.hq.map(|i| i.index()),
            leader: value.leader.map(|i| i.index()),
            assets: value.assets,
            notes: value.notes,
            general: value.general,
        }
    }
}
