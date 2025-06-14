use std::path::Path;

use anyhow::{Result, Ok, anyhow};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{action::{Action, ActionNode}, app::load_from_pot, app_display::DisplayTable, district::District, faction::{Faction, FactionStore}, managed_list::ManagedList, person::Person};

const DATA_EXTENSION: &str = "pot";


#[allow(dead_code)]
// Default should be empty
#[derive(Default)]
pub struct AppData {
    persons: ManagedList<Person>,
    districts: ManagedList<District>,
    factions: ManagedList<Faction>,
}

#[allow(dead_code)]
impl AppData {
    /// Takes a node, and creates the node that will reverse the actions taken
    pub fn do_action ( &mut self, actions: &ActionNode ) -> Result<ActionNode> {
        use Action::*;

        let mut return_node = ActionNode::default();

        // todo: do we want to explicitly pop so that we dismantle as we go?
        // what about when we fail?
        for action in actions {
            // do action
            //    can we fail?
            match action {
                DistrictAdd( district ) => {
                    if let Some(district_ref) = self.districts.add(district) {
                        return_node.push_back(DistrictRemove(district_ref));
                    }  // silently ignore if can't be added
                }

                DistrictRemove( district_ref ) => {
                    if let Some(district) = self.districts.remove(district_ref) {
                        return_node.push_back(DistrictAdd(district));
                    }  // silently ignore if this wasn't in the list when we removed it
                }

                DistrictReplace( district_ref, district ) => {
                    if let Some(old_district) = self.districts.replace(district_ref, district.clone()) {
                        return_node.push_back(DistrictReplace(district_ref.clone(), old_district));
                    }  // silently ignore if no replacement was possible?
                }

                PersonAdd( person ) => {
                    if let Some(person_ref) = self.persons.add(person) {
                        return_node.push_back(PersonRemove(person_ref));
                    }  // silently ignore if can't be added
                }

                PersonRemove( person_ref ) => {
                    if let Some(person) = self.persons.remove(person_ref) {
                        return_node.push_back(PersonAdd(person));
                    }  // silently ignore if this wasn't in the list when we removed it
                }

                PersonReplace( person_ref, person ) => {
                    if let Some(old_person) = self.persons.replace(person_ref, person.clone()) {
                        return_node.push_back(PersonReplace(person_ref.clone(), old_person));
                    }  // silently ignore if no replacement was possible?
                }

                FactionAdd( faction ) => {
                    if let Some(faction_ref) = self.factions.add(faction) {
                        return_node.push_back(FactionRemove(faction_ref));
                    }  // silently ignore if can't be added
                }

                FactionRemove( faction_ref ) => {
                    if let Some(faction) = self.factions.remove(faction_ref) {
                        return_node.push_back(FactionAdd(faction));
                    }  // silently ignore if this wasn't in the list when we removed it
                }

                FactionReplace( faction_ref, faction ) => {
                    if let Some(old_faction) = self.factions.replace(faction_ref, faction.clone()) {
                        return_node.push_back(FactionReplace(faction_ref.clone(), old_faction));
                    }  // silently ignore if no replacement was possible?
                }
            }
        }
        // fill return node
        Ok(return_node)
    }

    // todo: precalc and cache this?
    pub fn persons_display_table ( &self ) -> DisplayTable {
        (&self.persons).into()
    }

    // todo: precalc and cache this?
    pub fn districts_display_table ( &self ) -> DisplayTable {
        (&self.districts).into()
    }

    // todo: precalc and cache this?
    pub fn factions_display_table ( &self ) -> DisplayTable {
        (&self.factions).into()
    }

    pub fn set_persons_sort ( &mut self, index: usize ) {
        self.persons.set_sorting(index);
    }

    pub fn set_districts_sort ( &mut self, index: usize ) {
        self.districts.set_sorting(index);
    }

    pub fn set_factions_sort ( &mut self, index: usize ) {
        self.factions.set_sorting(index);
    }

    pub fn save_to_file ( &self ) {

    }

    pub fn load_from_file ( file_path: &Path ) -> Result<AppData> {
        load_save_data(&file_path.with_extension(DATA_EXTENSION))
    }

    // todo: actually we'd want to add this to the current data, and use the file name (_file_path: &Path)
    pub fn import_from_json ( &mut self ) -> Result<()> {
        #[derive(Deserialize)]
        struct ImportData {
            persons: Vec<Person>,
            districts: Vec<District>,
            factions: Vec<FactionStore>,
        }

        let data = include_str!("../test_data/test1.json");

        let import: ImportData = serde_json::from_str(data)?;
        debug!("imported {} people, {} districts, {} factions",
                import.persons.len(), import.districts.len(), import.factions.len());

        let district_add: ActionNode = import.districts.into_iter().map(|d| {
            Action::DistrictAdd(d)
        }).collect();

        let person_add = import.persons.into_iter().map(|p| {
            Action::PersonAdd(p)
        }).collect();

        // does this have to be done after the other two are in place, so we can make the right refs?
        let faction_add = import.factions.into_iter().map(|f| {
            Action::FactionAdd(f.into())
        }).collect();

        if let Err(err) = self.do_action(&district_add) {
            error!("unable to add districts: {}", err);
        }

        if let Err(err) = self.do_action(&person_add) {
            error!("unable to add persons: {}", err);
        }

        if let Err(err) = self.do_action(&faction_add) {
            error!("unable to add factions: {}", err);
        }

        Ok(())
    }
}

fn load_save_data ( file_path: &Path ) -> Result<AppData> {
    // load save data 1
    let data = load_from_pot::<SaveData1>(file_path)?;
    if data.validate() {
        // convert to AppData
        let ret = data.into();
        Ok(ret)
    } else {
        Err(anyhow!("unable to validate save data"))
    }
}

// ====================
// SaveData1
const SAVE1_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct SaveData1 {
    save_version: u16,
}

impl SaveData1 {
    fn validate ( &self ) -> bool {
        self.save_version == SAVE1_VERSION
    }
}

impl From<SaveData1> for AppData {
    fn from(_value: SaveData1) -> Self {
        AppData {
            persons: ManagedList::<Person>::default(),  // TODO
            districts: ManagedList::<District>::default(),  // TODO
            factions: ManagedList::<Faction>::default(),  // TODO
        }
    }
}

impl From<&AppData> for SaveData1 {
    fn from(_value: &AppData) -> Self {
        SaveData1 {
            save_version: SAVE1_VERSION,
        }
    }
}


// ----------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DataIndex {
    Nothing,
    FactionIndex(usize),
    PersonIndex(usize),
    DistrictIndex(usize),
}

impl DataIndex {
    pub fn index ( &self ) -> Option<usize> {
        match self {
            DataIndex::DistrictIndex(i) |
            DataIndex::FactionIndex(i) |
            DataIndex::PersonIndex(i) => Some(*i),

            _ => None
        }
    }
}
