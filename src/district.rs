
use serde::{Deserialize, Serialize};

use crate::{app_data::DataIndex, managed_list::Named};



#[allow(dead_code)]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
