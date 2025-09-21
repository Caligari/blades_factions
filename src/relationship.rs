use std::collections::BTreeMap;

use crate::app_data::{AppData, DataIndex};

// This cannot be based on GenericRef<> because that would make it impossible
// to use the one struct to make relationships between categories of items
#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorPair(DataIndex, DataIndex);

// For storing, we want to turn the DataIndexes into names
// For loading, we want to turn names into DataIndexes
//
// data indexes can get you to items, but if either item is None, the Pair should be ignored or removed

#[allow(dead_code)]
impl ActorPair {
    pub fn new(actor1: DataIndex, actor2: DataIndex) -> ActorPair {
        assert_ne!(actor1, actor2);
        assert_ne!(actor1, DataIndex::Nothing);
        assert_ne!(actor2, DataIndex::Nothing);

        if actor1 < actor2 {
            ActorPair(actor1, actor2)
        } else {
            ActorPair(actor2, actor1)
        }
    }

    /// Requires access to the master data, as an ActionPair is a pair of hypothetical references
    pub fn is_valid(&self, app_data: &AppData) -> bool {
        app_data.data_index_valid(self.0) && app_data.data_index_valid(self.1)
    }

    pub fn pair_with(&self, data_index: DataIndex) -> Option<DataIndex> {
        if self.0 == data_index {
            Some(self.1)
        } else if self.1 == data_index {
            Some(self.0)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub struct Relationships {
    table: BTreeMap<ActorPair, Relationship>, // ?? Btree or Hash
}

#[allow(dead_code)]
impl Relationships {
    pub fn my_relationships(&self, my_data_index: DataIndex) -> MyRelationships {
        let table: BTreeMap<DataIndex, Relationship> = self
            .table
            .iter()
            .filter_map(|(k, r)| k.pair_with(my_data_index).as_ref().map(|d| (*d, r.clone())))
            .collect();
        MyRelationships {
            my_data_index,
            table,
        }
    }
}

// Need to get list of all Relationships with a given DataIndex/actor
// That list will have only the actor info for the other party

#[allow(dead_code)]
pub struct MyRelationships {
    my_data_index: DataIndex,
    table: BTreeMap<DataIndex, Relationship>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Relationship {
    attitude: Attitude, // Does this need names?
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Attitude {
    Minus3 = -3,
    Minus2,
    Minus1,
    #[default]
    Zero,
    Plus1,
    Plus2,
    Plus3,
}
