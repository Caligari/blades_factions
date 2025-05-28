use std::{collections::BTreeMap, rc::Rc};



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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistrictIndex {
    name: String,
    index: usize,
}

#[allow(dead_code)]
impl DistrictIndex {
    pub fn index ( &self ) -> usize {
        self.index
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistrictList {
    factions: Vec<District>,  // ?maybe something that only grows?
    factions_index: BTreeMap<String, Rc<DistrictIndex>>,
}
