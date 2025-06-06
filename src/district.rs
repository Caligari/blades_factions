use std::{collections::BTreeMap, sync::Arc};

use eframe::egui::mutex::RwLock;

use crate::{app_data::DataIndex, managed_list::Named};



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
}

impl Named for District {
    fn name ( &self ) -> &str {
        &self.name
    }

    fn make_data_index ( index: usize ) -> DataIndex {
        DataIndex::DistrictIndex(index)
    }

    fn fetch_data_index ( index: DataIndex ) -> Option<usize> {
        match index {
            DataIndex::DistrictIndex( ind ) => Some(ind),
            _ => None,
        }
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
