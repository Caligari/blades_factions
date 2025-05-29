use std::{collections::BTreeMap, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::app_data::DataIndex;





#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Person {
    name: String,
    description: String,
    // characteristics strings
    notes: String,
    // connections?
}

pub type PersonRef = Rc<PersonIndex>;

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
#[derive(Debug, Clone)]
pub struct PersonList {
    factions: Vec<Person>,  // ?maybe something that only grows?
    factions_index: BTreeMap<String, Rc<PersonIndex>>,
}

