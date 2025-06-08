
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, managed_list::Named};





#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Person {
    name: String,
    description: String,
    // characteristics strings
    notes: String,
    // connections?
    // faction?
    // home?
}

impl Named for Person {
    fn name ( &self ) -> &str {
        &self.name
    }

    fn make_data_index ( index: usize ) -> DataIndex {
        DataIndex::PersonIndex(index)
    }

    fn fetch_data_index ( index: DataIndex ) -> Option<usize> {
        match index {
            DataIndex::PersonIndex( ind ) => Some(ind),
            _ => None,
        }
    }
}
