use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use anyhow::{Result, Ok, anyhow};
use eframe::egui::mutex::RwLock;

use crate::{app_data::DataIndex, district::District, faction::Faction, person::Person};





#[allow(dead_code)]
pub type FactionRef = Arc<RwLock<NamedIndex<Faction>>>;

#[allow(dead_code)]
pub type PersonRef = Arc<RwLock<NamedIndex<Person>>>;

#[allow(dead_code)]
pub type DistrictRef = Arc<RwLock<NamedIndex<District>>>;


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NamedIndex<T: Clone + Named> {
    name: String,
    // index: usize,
    index: DataIndex,
    typ: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Clone + Named> NamedIndex<T> {
    pub fn name ( &self ) -> &str {
        &self.name
    }

    pub fn index ( &self ) -> DataIndex {
        self.index
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct ManagedList<T: Clone + Named> {
    list: Vec<T>,
    list_index: BTreeMap<String, Arc<RwLock<NamedIndex<T>>>>,
}

#[allow(dead_code)]
impl<T: Clone + Named> ManagedList<T> {
    pub fn add ( &mut self, item: &T ) -> Result<Arc<RwLock<NamedIndex<T>>>> {
        if !self.list_index.contains_key(item.name()) {
            let name = item.name().to_string();
            let index = T::make_data_index(self.list.len());
            self.list.push(item.clone());
            let named_index = Arc::new(RwLock::new(NamedIndex { name: name.clone(), index, typ: PhantomData }));
            self.list_index.insert(name, named_index.clone());
            Ok(named_index)
        } else { Err(anyhow!("key already present in list")) }
    }

}


// -------------------------------
// Named

pub trait Named {
    fn name ( &self ) -> &str;
    fn make_data_index ( index: usize ) -> DataIndex;
}
