use std::{collections::BTreeMap, sync::Arc};

use eframe::egui::mutex::RwLock;
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, managed_list::Named};





#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Person {
    name: String,
    description: String,
    // characteristics strings
    notes: String,
    // connections?
}

impl Named for Person {
    fn name ( &self ) -> &str {
        &self.name
    }
}


pub type PersonRef = Arc<RwLock<PersonIndex>>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PersonIndex {
    name: String,
    index: usize,
}

#[allow(dead_code)]
impl PersonIndex {
    pub fn index ( &self ) -> DataIndex {
        DataIndex::PersonIndex(self.index)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct PersonList {
    factions: Vec<Person>,  // ?maybe something that only grows?
    factions_index: BTreeMap<String, PersonRef>,
}

