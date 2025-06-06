use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use anyhow::{Result, Ok, anyhow};
use eframe::egui::mutex::RwLock;
use log::{info, warn};

use crate::{app_data::DataIndex, district::District, faction::Faction, person::Person};





#[derive(Clone)]
pub struct GenericRef<T: Clone + Named> ( Arc<RwLock<NamedIndex<T>>> );

impl<T: Clone + Named> GenericRef<T> {
    pub fn has_index ( &self ) -> bool {
        !matches!(self.0.read().index, DataIndex::Nothing)
    }

    pub fn index ( &self ) -> Option<usize> {
        self.0.read().index.index()
    }
}

#[allow(dead_code)]
pub type FactionRef = GenericRef<Faction>;

#[allow(dead_code)]
pub type PersonRef = GenericRef<Person>;

#[allow(dead_code)]
pub type DistrictRef = GenericRef<District>;


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NamedIndex<T: Clone + Named> {
    name: String,
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
    list_index: BTreeMap<String, GenericRef<T>>,
}

#[allow(dead_code)]
impl<T: Clone + Named> ManagedList<T> {
    pub fn len ( &self ) -> usize {
        self.list.len()
    }

    pub fn add ( &mut self, item: &T ) -> Result<GenericRef<T>> {
        if !self.list_index.contains_key(item.name()) {
            let name = item.name().to_string();
            let index = T::make_data_index(self.list.len());
            self.list.push(item.clone());
            let named_index = GenericRef(
                Arc::new(RwLock::new(NamedIndex { name: name.clone(), index, typ: PhantomData }))
            );
            self.list_index.insert(name, named_index.clone());
            Ok(named_index)
        } else { Err(anyhow!("key already present in list")) }
    }

    // Should the list have a lock on it??
    pub fn remove ( &mut self, named_index: &GenericRef<T> ) -> Result<Option<T>> {
        if named_index.has_index() {
            let Some(index) = named_index.index()
                else { return Err(anyhow!("asked to remove incorrect index from managed list")); };

            // process the list index, changing the indexes for the entries after this one
            for (s, i) in self.list_index.iter() {
                info!("looking at [{s}]");
                let mut ind = i.0.write();
                if let Some(cur_i) = ind.index.index() {
                    info!("processing index for {cur_i}");
                    if cur_i > index {
                        info!("decrementing");
                        ind.index = T::make_data_index(cur_i - 1);
                    } else if cur_i == index {
                        info!("will remove");
                        ind.index = DataIndex::Nothing;
                    } else { info!("ignoring"); }  // else it will not need to change
                }  // else we do not need to change somethning which points to no data
            }

            // remove the element
            info!("removing now");
            let ret = self.list.remove(index);
            // return the removed element
            Ok(Some(ret))
        } else {
            warn!("asked to remove empty index");
            Ok(None)
        }
    }

    // todo - replace

    // todo - fetch
    pub fn fetch ( &self, _index: &GenericRef<T> ) -> Result<Option<T>> {
        Ok(None)
    }
}


// -------------------------------
// Named

#[allow(dead_code)]
pub trait Named {
    fn name ( &self ) -> &str;
    fn make_data_index ( index: usize ) -> DataIndex;
    fn fetch_data_index ( index: DataIndex ) -> Option<usize>;
}

#[cfg(test)]
mod tests {

    use std::fs;

    use log::LevelFilter;

    use crate::{district::District, managed_list::{ManagedList, Named}};


    #[test]
    fn add_managed_list () {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        if let Err(e) = m_list.add(&item1) {
            println!("add_managed_list: error on add item1 - {e}");  // do we need this log message?
            panic!("error on add item1: {e}");
        }
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        if let Err(e) = m_list.add(&item2) {
            println!("add_managed_list: error on add item2 - {e}");  // do we need this log message?
            panic!("error on add item2: {e}");
        }
        assert_eq!(m_list.len(), 2);
    }

    #[test]
    fn remove_managed_list () {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        let _item1_ref = match m_list.add(&item1) {
            Err(e) => {
                println!("remove_managed_list: error on add item1 - {e}");  // do we need this log message?
                panic!("error on add item1: {e}");
            },

            Ok(ret) => ret
        };
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        let item2_ref = match m_list.add(&item2) {
            Err(e) => {
                println!("remove_managed_list: error on add item2 - {e}");  // do we need this log message?
                panic!("error on add item2: {e}");
            },

            Ok(ret) => ret
        };
        assert_eq!(m_list.len(), 2);

        let item3 = District::new("Test3");
        let _item3_ref = match m_list.add(&item3) {
            Err(e) => {
                println!("remove_managed_list: error on add item3 - {e}");  // do we need this log message?
                panic!("error on add item3: {e}");
            },

            Ok(ret) => ret
        };
        assert_eq!(m_list.len(), 3);

        let remove1 = match m_list.remove(&item2_ref) {
            Err(e) => {
                println!("remove_managed_list: error on remove item2 - {e}");  // do we need this log message?
                panic!("error on remove item2: {e}");
            },

            Ok(ret) => ret
        };
        assert!(remove1.is_some());
        assert_eq!(remove1.unwrap().name(), "Test2");
        assert_eq!(m_list.len(), 2);

        let remove2 = match m_list.remove(&item2_ref) {
            Err(e) => {
                println!("remove_managed_list: error on remove item2 again - {e}");  // do we need this log message?
                panic!("error on remove item2 again: {e}");
            },

            Ok(ret) => ret
        };
        assert!(remove2.is_none());
        assert_eq!(m_list.len(), 2);
    }


    #[allow(dead_code)]
    fn setup_logger ( ) -> Result<(), fern::InitError> {
        const LOG_FILE: &str = "factions_test_output.log";
        let _ = fs::remove_file(LOG_FILE);  // !! ignoring possible real errors
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{:.5}][{}]: {}",
                    // "[{}][{}] {}",
                        // "[{}]:[{}][{}] {}",
                        // humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(LevelFilter::Debug)
            .level_for(module_path!(), LevelFilter::Debug)
            .chain(std::io::stdout())
            .chain(fern::log_file(LOG_FILE)?)
            .apply()?;
        Ok(())
    }

}