use std::{collections::BTreeMap, sync::Arc};

use eframe::egui::mutex::RwLock;

use crate::app_data::DataIndex;



#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct District {
    name: String,
    // ? is there other data to store?
}

#[allow(dead_code)]
impl District {
    pub fn new ( name: &str ) -> Self {
        District {
            name: name.to_string(),
        }
    }

    pub fn name ( &self ) -> &str {
        &self.name
    }
}

pub type DistrictRef = Arc<RwLock<DistrictIndex>>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistrictIndex {
    name: String,
    index: usize,
}

#[allow(dead_code)]
impl DistrictIndex {
    pub fn index ( &self ) -> DataIndex {
        DataIndex::DistrictIndex(self.index)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct DistrictList {
    factions: Vec<District>,  // ?maybe something that only grows?
    factions_index: BTreeMap<String, DistrictRef>,
}
