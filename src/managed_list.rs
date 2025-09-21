use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use eframe::egui::{RichText, mutex::RwLock};
use log::{debug, info, warn};

use crate::{
    app_data::DataIndex,
    app_display::NewStringStatus,
    district::{District, DistrictStore},
    faction::{Faction, FactionStore},
    person::{Person, PersonStore1, PersonStore2},
    sorting::Sorting,
};

#[derive(Clone)]
pub struct GenericRef<T: Clone + Named>(Arc<RwLock<NamedIndex<T>>>);

impl<T: Clone + Named> GenericRef<T> {
    pub fn has_index(&self) -> bool {
        !matches!(self.0.read().index, DataIndex::Nothing)
    }

    // not public because noone should cache this, even accidentially
    /// Returns a transient index value that is only valid right now
    fn index(&self) -> Option<usize> {
        self.0.read().index.index()
    }

    /// Use extreme caution with the return value, as it can be changed later and you will not know
    pub fn data_index(&self) -> DataIndex {
        self.0.read().index
    }

    // pub fn index_type ( &self ) -> IndexType {
    //     self.0.read().index.into()
    // }

    pub fn name(&self) -> Option<String> {
        self.0.read().name().map(|n| n.to_string())
    }

    pub fn display_name(&self) -> Option<String> {
        self.0.read().display_name().map(|n| n.to_string())
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

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct GenericRefList<T: Clone + Named> {
    list: Vec<GenericRef<T>>, // should this be a set? Not worth the pain? Requires ordering to not change, which we can't guarantee
    new: Option<String>,
    hovered: Option<String>,
}

#[allow(dead_code)]
impl<T: Clone + Named> GenericRefList<T> {
    pub fn from_list(input_list: Vec<GenericRef<T>>) -> Self {
        GenericRefList::<T> {
            list: input_list,
            new: None,
            hovered: None,
        }
    }

    pub fn list(&self) -> &Vec<GenericRef<T>> {
        &self.list
    }

    /// This silently ignores duplicates
    pub fn push(&mut self, item: GenericRef<T>) {
        if let Some(new_name) = item.name()
            && !self.list.iter().any(|r| r.0.read().name == new_name)
        {
            self.list.push(item);
        }
    }

    pub fn swap_remove(&mut self, item_name: &str) {
        // ?? should return success?
        let Some(index) = self.list.iter().position(|r| r.0.read().name == item_name) else {
            warn!("failed to remove item >{item_name}> from GenericRefList");
            return;
        };
        self.list.swap_remove(index);
    }

    pub fn new_name(&self) -> Option<&str> {
        self.new.as_deref()
    }

    pub fn hovered_name(&self) -> Option<&str> {
        self.hovered.as_deref()
    }

    pub fn set_new(&mut self, name: Option<&str>) {
        self.new = name.map(|n| n.to_string());
    }

    pub fn set_hovered(&mut self, name: Option<String>) {
        self.hovered = name;
    }
}

impl<T: Clone + Named + PartialEq> PartialEq for GenericRefList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list
    }
}

#[allow(dead_code)]
pub type FactionRefList = GenericRefList<Faction>;

#[allow(dead_code)]
pub type PersonRefList = GenericRefList<Person>;

#[allow(dead_code)]
pub type DistrictRefList = GenericRefList<District>;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NamedIndex<T: Clone + Named> {
    name: String,
    display_name: String,
    index: DataIndex,
    typ: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Clone + Named> NamedIndex<T> {
    pub fn name(&self) -> Option<&str> {
        if !matches!(self.index, DataIndex::Nothing) {
            Some(&self.name)
        } else {
            None
        }
    }

    pub fn display_name(&self) -> Option<&str> {
        if !matches!(self.index, DataIndex::Nothing) {
            Some(&self.display_name)
        } else {
            None
        }
    }

    fn index(&self) -> DataIndex {
        self.index
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct ManagedList<T: Clone + Named> {
    list: Vec<Option<T>>,
    list_index: BTreeMap<String, GenericRef<T>>,
    sorting: Sorting,
}

#[allow(dead_code)]
impl<T: Clone + Named> ManagedList<T> {
    pub fn len(&self) -> usize {
        self.list.len() // includes removed items
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty() // includes removed items
    }

    /// Returns the reference to the new item
    pub fn add(&mut self, item: &T) -> Option<GenericRef<T>> {
        let name = item.name().to_string();
        let display_name = item.display_name();
        if !self.list_index.contains_key(&name) {
            let index = T::make_data_index(self.list.len());
            self.list.push(Some(item.clone()));
            let named_index = GenericRef(Arc::new(RwLock::new(NamedIndex {
                name: name.clone(),
                display_name,
                index,
                typ: PhantomData,
            })));
            self.list_index.insert(name, named_index.clone());
            Some(named_index)
        } else {
            warn!("key {name} already present in list, during add");
            None
        }
    }

    // Should the list have a lock on it??
    /// Returns the old item which was removed, if it was present
    pub fn remove(&mut self, named_index: &mut GenericRef<T>) -> Option<T> {
        if named_index.has_index() {
            let Some(index) = named_index.index() else {
                panic!("asked to remove incorrect index from managed list");
            };

            info!("(not) removing {:?}", named_index.data_index());

            let ret = if let Some(Some(val)) = self.list.get(index) {
                Some(val.clone())
            } else {
                None
            };
            self.list[index] = None; // void the item

            // update the reference
            let mut ind = named_index.0.write();
            ind.index = DataIndex::Nothing;
            ind.name = "<Removed>".to_owned();

            ret
        } else {
            warn!("asked to remove empty index");
            None
        }
    }

    /// Returns the old item, if something was replaced
    /// Note: the reference name is updated as well
    pub fn replace(&mut self, index: &GenericRef<T>, new_item: T) -> Option<T> {
        if index.has_index() {
            let Some(ind) = index.index() else {
                unreachable!("no index found despite having an index (in replace)");
            };
            let Some(Some(old_item)) = self.list.get(ind).cloned()
            // technically this should always return Some
            else {
                unreachable!("unable to find managed_list item with functioning index");
            };
            let new_name = new_item.name();
            let old_name = old_item.name();
            let same_name = new_name == old_name;
            let new_display = new_item.display_name();
            let old_display: String = old_item.display_name();
            let same_display = new_display == old_display;

            if !same_display {
                index.0.write().display_name = new_display;
            }

            if !same_name && self.list_index.contains_key(new_name) {
                warn!("unable to replace {old_name} with {new_name}, as it is alreay present");
                None
            } else {
                if !same_name {
                    // update name in index
                    index.0.write().name = new_name.to_owned();

                    debug!("replacing {old_name} with {new_name}");

                    // insert into btree with new_name and index.clone()
                    if self
                        .list_index
                        .insert(new_name.to_string(), index.clone())
                        .is_none()
                    {
                        // returns an option<ref>
                        // remove from btree based on index.0.name()
                        self.list_index.remove(old_name); // returns option<ref>
                    } else {
                        unreachable!("updating list_index returned existing index!");
                    }
                }

                // replace content
                self.list[ind] = Some(new_item);

                // return old item
                Some(old_item)
            }
        } else {
            warn!("asked to replace an empty item"); // should this do an add using the old reference?
            None
        }
    }

    /// Returns the reference to the named item, if it exists
    pub fn find(&self, name: &str) -> Option<GenericRef<T>> {
        self.list_index.get(name).cloned()
    }

    /// Returns a reference to the existing item, if it is present
    pub fn fetch(&self, index: &GenericRef<T>) -> Option<&T> {
        let index = index.index()?;
        self.fetch_with_index(index)
    }

    pub fn item_ref_list(&self) -> Vec<(GenericRef<T>, &T)> {
        self.list_index
            .iter()
            .filter_map(|(_st, re)| {
                if re.has_index() {
                    self.fetch(re).map(|item| (re.clone(), item))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_sorting(&self) -> Sorting {
        self.sorting
    }

    pub fn set_sorting(&mut self, index: usize) {
        self.sorting.set_field(index);
    }

    pub fn names_sorted(&self) -> Vec<String> {
        let mut list_copy: Vec<String> = self.list_index.keys().cloned().collect();
        list_copy.sort(); // ?? do we need a more sophisticated sort? For People Names, for example
        list_copy
    }

    /// Use care with this
    pub fn fetch_with_index(&self, index: usize) -> Option<&T> {
        let Some(ret) = self.list.get(index) else {
            unreachable!("trying to access indexed item that is not present");
        };
        ret.as_ref()
    }
}

impl From<&ManagedList<Person>> for Vec<PersonStore2> {
    fn from(value: &ManagedList<Person>) -> Self {
        value
            .list
            .iter()
            .filter_map(|maybe_p| maybe_p.as_ref().map(PersonStore2::from))
            .collect()
    }
}

impl From<&ManagedList<Person>> for Vec<PersonStore1> {
    fn from(value: &ManagedList<Person>) -> Self {
        value
            .list
            .iter()
            .filter_map(|maybe_p| maybe_p.as_ref().map(PersonStore1::from))
            .collect()
    }
}

impl From<&ManagedList<District>> for Vec<DistrictStore> {
    fn from(value: &ManagedList<District>) -> Self {
        value
            .list
            .iter()
            .filter_map(|maybe_p| maybe_p.as_ref().map(DistrictStore::from))
            .collect()
    }
}

impl From<&ManagedList<Faction>> for Vec<FactionStore> {
    fn from(value: &ManagedList<Faction>) -> Self {
        value
            .list
            .iter()
            .filter_map(|maybe_p| maybe_p.as_ref().map(FactionStore::from))
            .collect()
    }
}

// -------------------------------
// Named

#[allow(dead_code)]
pub trait Named {
    fn name(&self) -> &str;
    fn display_name(&self) -> String;
    fn make_data_index(index: usize) -> DataIndex;
    fn fetch_data_index(index: DataIndex) -> Option<usize>;
    fn display_fields(&self) -> Vec<String>;
    fn display_headings() -> Vec<RichText>;
}

// --------------------------------
// StringList

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct StringList {
    list: Vec<String>, // should this be a set? Not worth the pain? Requires ordering to not change, which we can't guarantee
    new: NewStringStatus,
    hovered: Option<String>,
}

#[allow(dead_code)]
impl StringList {
    pub fn from_list(input_list: Vec<String>) -> Self {
        StringList {
            list: input_list,
            new: NewStringStatus::NoItem,
            hovered: None,
        }
    }

    pub fn list(&self) -> &Vec<String> {
        &self.list
    }

    /// This silently ignores duplicates
    pub fn push(&mut self, item: String) {
        if !self.list.contains(&item) {
            self.list.push(item);
        }
    }

    pub fn swap_remove(&mut self, item_name: &str) {
        // ?? should return success?
        let Some(index) = self.list.iter().position(|r| r == item_name) else {
            warn!("failed to remove item >{item_name}> from StringList");
            return;
        };
        self.list.swap_remove(index);
    }

    pub fn new_name(&mut self) -> &mut NewStringStatus {
        &mut self.new
    }

    pub fn hovered_name(&self) -> Option<&str> {
        self.hovered.as_deref()
    }

    pub fn set_new(&mut self, name: NewStringStatus) {
        self.new = name;
    }

    pub fn set_hovered(&mut self, name: Option<String>) {
        self.hovered = name;
    }
}

impl PartialEq for StringList {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use log::LevelFilter;

    use crate::{
        district::District,
        managed_list::{ManagedList, Named},
    };

    // TODO: add tests with replace and find

    #[test]
    fn basic_managed_list() {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        let Some(item1_ref) = m_list.add(&item1) else {
            panic!("error on add item1");
        };
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        let Some(_item2_ref) = m_list.add(&item2) else {
            panic!("error on add item2");
        };
        assert_eq!(m_list.len(), 2);

        let found1 = m_list.fetch(&item1_ref);
        assert!(found1.is_some(), "found item 1 is empty");
        assert_eq!(found1.unwrap().name(), "Test1");

        let Some(found2_ref) = m_list.find("Test2") else {
            panic!("unable to find item2 in list");
        };
        let found2 = m_list.fetch(&found2_ref);
        assert!(found2.is_some(), "found item 2 is empty");
        assert_eq!(found2.unwrap().name(), "Test2");

        let new_item1 = District::new("New1");
        let Some(old1) = m_list.replace(&item1_ref, new_item1) else {
            panic!("unable to replace item 1");
        };
        assert_eq!(old1.name(), "Test1");
        assert!(m_list.fetch(&item1_ref).is_some());
        assert_eq!(m_list.fetch(&item1_ref).unwrap().name(), "New1");
    }

    #[test]
    fn remove_managed_list() {
        // setup_logger().expect("log did not start");
        let mut m_list = ManagedList::<District>::default();

        let item1 = District::new("Test1");
        let Some(item1_ref) = m_list.add(&item1) else {
            panic!("error on add item1");
        };
        assert_eq!(m_list.len(), 1);

        let item2 = District::new("Test2");
        let Some(mut item2_ref) = m_list.add(&item2) else {
            panic!("error on add item2");
        };
        assert_eq!(m_list.len(), 2);

        let item3 = District::new("Test3");
        let Some(item3_ref) = m_list.add(&item3) else {
            panic!("error on add item3");
        };
        assert_eq!(m_list.len(), 3);

        let remove1 = m_list.remove(&mut item2_ref);
        assert!(remove1.is_some());
        assert_eq!(remove1.unwrap().name(), "Test2");
        assert_eq!(m_list.len(), 3); // !! used to be 2

        let remove2 = m_list.remove(&mut item2_ref);
        assert!(remove2.is_none());
        assert_eq!(m_list.len(), 3); // !! used to be 2

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
    fn setup_logger() -> Result<(), fern::InitError> {
        const LOG_FILE: &str = "factions_test_output.log";
        let _ = fs::remove_file(LOG_FILE); // !! ignoring possible real errors
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
