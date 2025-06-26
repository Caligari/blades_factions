use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use eframe::egui::{mutex::RwLock, RichText};
use log::{debug, info, warn};

use crate::{app_data::DataIndex, district::District, faction::Faction, person::Person, sorting::Sorting};





#[derive(Clone)]
pub struct GenericRef<T: Clone + Named> ( Arc<RwLock<NamedIndex<T>>> );

impl<T: Clone + Named> GenericRef<T> {
    pub fn has_index ( &self ) -> bool {
        !matches!(self.0.read().index, DataIndex::Nothing)
    }

    // not public because noone should cache this, even accidentially
    /// Returns a transient index value that is only valid right now
    fn index ( &self ) -> Option<usize> {
        self.0.read().index.index()
    }

    /// Use extreme caution with the return value, as it can be changed later and you will not know
    pub fn data_index ( &self ) -> DataIndex {
        self.0.read().index
    }

    // pub fn index_type ( &self ) -> IndexType {
    //     self.0.read().index.into()
    // }

    pub fn name ( &self ) -> Option<String> {
        self.0.read().name().map(|n| n.to_string())
    }
}

impl<T: Clone + Named + PartialEq> PartialEq for GenericRef<T> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.read().clone();
        let b = other.0.read().clone();
        a == b
    }
}

#[allow(dead_code)]
pub type FactionRef = GenericRef<Faction>;

#[allow(dead_code)]
pub type PersonRef = GenericRef<Person>;

#[allow(dead_code)]
pub type DistrictRef = GenericRef<District>;

// pub enum IndexType {
//     Nothing,
//     District,
//     Person,
//     Faction,
// }

// impl From <DataIndex> for IndexType {
//     fn from ( value: DataIndex ) -> Self {
//         use DataIndex::*;
//         match value {
//             Nothing => IndexType::Nothing,
//             DistrictIndex (..) => IndexType::District,
//             PersonIndex (..) => IndexType::Person,
//             FactionIndex(..) => IndexType::Faction,
//         }
//     }
// }

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NamedIndex<T: Clone + Named> {
    name: String,
    index: DataIndex,
    typ: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Clone + Named> NamedIndex<T> {
    pub fn name ( &self ) -> Option<&str> {
        if !matches!(self.index, DataIndex::Nothing) {
            Some(&self.name)
        } else { None }
    }

    fn index ( &self ) -> DataIndex {
        self.index
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct ManagedList<T: Clone + Named> {
    list: Vec<T>,
    list_index: BTreeMap<String, GenericRef<T>>,
    sorting: Sorting,
}

#[allow(dead_code)]
impl<T: Clone + Named> ManagedList<T> {
    pub fn len ( &self ) -> usize {
        self.list.len()
    }

    /// Returns the reference to the new item
    pub fn add ( &mut self, item: &T ) -> Option<GenericRef<T>> {
        let name = item.name().to_string();
        if !self.list_index.contains_key(&name) {
            let index = T::make_data_index(self.list.len());
            self.list.push(item.clone());
            let named_index = GenericRef(
                Arc::new(RwLock::new(NamedIndex { name: name.clone(), index, typ: PhantomData }))
            );
            self.list_index.insert(name, named_index.clone());
            Some(named_index)
        } else {
            warn!("key {name} already present in list, during add");
            None
        }
    }

    // Should the list have a lock on it??
    /// Returns the old item which was removed, if it was present
    pub fn remove ( &mut self, named_index: &GenericRef<T> ) -> Option<T> {
        if named_index.has_index() {
            let Some(index) = named_index.index()
                else { panic!("asked to remove incorrect index from managed list"); };

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
                        ind.name = "<Removed>".to_owned();
                    } else { info!("ignoring"); }  // else it will not need to change
                }  // else we do not need to change somethning which points to no data
            }

            // remove the element
            // return the removed element
            info!("removing now");
            Some(self.list.remove(index))
        } else {
            warn!("asked to remove empty index");
            None
        }
    }

    /// Returns the old item, if something was replaced
    /// Note: the reference name is updated as well
    pub fn replace ( &mut self, index: &GenericRef<T>, new_item: T ) -> Option<T> {
        if index.has_index() {
            let Some(ind) = index.index()
                else { unreachable!("no index found despite having an index (in replace)"); };
            let Some(old_item) = self.list.get(ind).cloned()  // technically this should always return Some
                else { unreachable!("unable to find managed_list item with functioning index"); };
            let new_name = new_item.name();
            let old_name = old_item.name();
            let same_name = new_name == old_name;

            if !same_name && self.list_index.contains_key(new_name) {
                warn!("unable to replace {old_name} with {new_name}, as it is alreay present");
                None
            } else {
                if !same_name {
                    // update name in index
                    index.0.write().name = new_name.to_owned();

                    debug!("replacing {old_name} with {new_name}");

                    // insert into btree with new_name and index.clone()
                    if self.list_index.insert(new_name.to_string(), index.clone()).is_none() { // returns an option<ref>
                        // remove from btree based on index.0.name()
                        self.list_index.remove(old_name);  // returns option<ref>
                    } else {
                        unreachable!("updating list_index returned existing index!");
                    }
                }

                // replace content
                self.list[ind] = new_item;

                // return old item
                Some(old_item)
            }
        } else {
            warn!("asked to replace an empty item");  // should this do an add using the old reference?
            None
        }
    }

    /// Returns the reference to the named item, if it exists
    pub fn find ( &self, name: &str ) -> Option<GenericRef<T>> {
        self.list_index.get(name).cloned()
    }

    /// Returns a reference to the existing item, if it is present
    pub fn fetch ( &self, index: &GenericRef<T> ) -> Option<&T> {
        let index = index.index()?;
        self.list.get(index)
    }

    pub fn item_ref_list ( &self ) -> Vec<(GenericRef<T>, &T)> {
        self.list_index.iter().filter_map(|(_st, re)| {
            if re.has_index() {
                self.fetch(re).map(|item| (re.clone(), item))
            } else { None }
        }).collect()
    }

    pub fn get_sorting ( &self ) -> Sorting {
        self.sorting
    }

    pub fn set_sorting ( &mut self, index: usize ) {
        self.sorting.set_field(index);
    }
}


// -------------------------------
// Named

#[allow(dead_code)]
pub trait Named {
    fn name ( &self ) -> &str;
    fn make_data_index ( index: usize ) -> DataIndex;
    fn fetch_data_index ( index: DataIndex ) -> Option<usize>;
    fn display_fields ( &self ) -> Vec<String>;
    fn display_headings ( ) -> Vec<RichText>;
}

#[cfg(test)]
mod tests {

    use std::fs;

    use log::LevelFilter;

    use crate::{district::District, managed_list::{ManagedList, Named}};

    // TODO: add tests with replace and find

    #[test]
    fn basic_managed_list () {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        let Some(item1_ref) = m_list.add(&item1)
            else { panic!("error on add item1"); };
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        let Some(_item2_ref) = m_list.add(&item2)
            else { panic!("error on add item2"); };
        assert_eq!(m_list.len(), 2);

        let found1 = m_list.fetch(&item1_ref);
        assert!(found1.is_some(), "found item 1 is empty");
        assert_eq!(found1.unwrap().name(), "Test1");

        let Some(found2_ref) = m_list.find("Test2")
            else { panic!("unable to find item2 in list"); };
        let found2 = m_list.fetch(&found2_ref);
        assert!(found2.is_some(), "found item 2 is empty");
        assert_eq!(found2.unwrap().name(), "Test2");

        let new_item1 = District::new("New1");
        let Some(old1) = m_list.replace(&item1_ref, new_item1)
            else { panic!("unable to replace item 1"); };
        assert_eq!(old1.name(), "Test1");
        assert!(m_list.fetch(&item1_ref).is_some());
        assert_eq!(m_list.fetch(&item1_ref).unwrap().name(), "New1");
    }

    #[test]
    fn remove_managed_list () {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        let Some(item1_ref) = m_list.add(&item1)
            else { panic!("error on add item1"); };
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        let Some(item2_ref) = m_list.add(&item2)
            else { panic!("error on add item2"); };
        assert_eq!(m_list.len(), 2);

        let item3 = District::new("Test3");
        let Some(item3_ref) = m_list.add(&item3)
            else { panic!("error on add item3"); };
        assert_eq!(m_list.len(), 3);

        let remove1 = m_list.remove(&item2_ref);
        assert!(remove1.is_some());
        assert_eq!(remove1.unwrap().name(), "Test2");
        assert_eq!(m_list.len(), 2);

        let remove2 = m_list.remove(&item2_ref);
        assert!(remove2.is_none());
        assert_eq!(m_list.len(), 2);

        let found1 = m_list.fetch(&item1_ref);
        assert!(found1.is_some(), "found item 1 is empty");
        assert_eq!(found1.unwrap().name(), "Test1");

        let found2 = m_list.fetch(&item2_ref);
        assert!(found2.is_none(), "found item 2 is not empty");

        let found3 = m_list.fetch(&item3_ref);
        assert!(found3.is_some(), "found item 3 is empty");
        assert_eq!(found3.unwrap().name(), "Test3");
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