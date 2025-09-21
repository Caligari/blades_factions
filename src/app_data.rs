use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use anyhow::{Ok, Result, anyhow};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{
    action::{Action, ActionNode},
    app::{MainView, load_from_json, load_from_save, save_to_json, save_to_save},
    app_display::DisplayTable,
    district::{District, DistrictStore},
    faction::{Faction, FactionStore},
    managed_list::{DistrictRef, FactionRef, ManagedList, Named, PersonRef},
    person::{Person, PersonStore1, PersonStore2},
};

pub const SAVE_EXTENSION: &str = "bfsav";
pub const JSON_EXTENSION: &str = "json";

#[allow(dead_code)]
// Default should be empty
#[derive(Default)]
pub struct AppData {
    loaded_from: Option<PathBuf>,
    persons: ManagedList<Person>,
    districts: ManagedList<District>,
    factions: ManagedList<Faction>,
}

#[allow(dead_code)]
impl AppData {
    /// Takes a node, and creates the node that will reverse the actions taken
    pub fn do_action(&mut self, actions: &mut ActionNode) -> Result<ActionNode> {
        use Action::*;

        let mut return_node = ActionNode::default();

        // todo: do we want to explicitly pop so that we dismantle as we go?
        // what about when we fail?
        for action in actions {
            // do action
            //    can we fail?
            match action {
                DistrictAdd(district) => {
                    if let Some(district_ref) = self.districts.add(district) {
                        return_node.push_back(DistrictRemove(district_ref));
                    } // silently ignore if can't be added
                }

                DistrictRemove(district_ref) => {
                    if let Some(district) = self.districts.remove(district_ref) {
                        return_node.push_back(DistrictAdd(district));
                    } // silently ignore if this wasn't in the list when we removed it
                }

                DistrictReplace(district_ref, district) => {
                    if let Some(old_district) =
                        self.districts.replace(district_ref, district.clone())
                    {
                        return_node.push_back(DistrictReplace(district_ref.clone(), old_district));
                    } // silently ignore if no replacement was possible?
                }

                PersonAdd(person) => {
                    if let Some(person_ref) = self.persons.add(person) {
                        return_node.push_back(PersonRemove(person_ref));
                    } // silently ignore if can't be added
                }

                PersonRemove(person_ref) => {
                    if let Some(person) = self.persons.remove(person_ref) {
                        return_node.push_back(PersonAdd(person));
                    } // silently ignore if this wasn't in the list when we removed it
                }

                PersonReplace(person_ref, person) => {
                    if let Some(old_person) = self.persons.replace(person_ref, person.clone()) {
                        return_node.push_back(PersonReplace(person_ref.clone(), old_person));
                    } // silently ignore if no replacement was possible?
                }

                FactionAdd(faction) => {
                    if let Some(faction_ref) = self.factions.add(faction) {
                        return_node.push_back(FactionRemove(faction_ref));
                    } // silently ignore if can't be added
                }

                FactionRemove(faction_ref) => {
                    if let Some(faction) = self.factions.remove(faction_ref) {
                        return_node.push_back(FactionAdd(faction));
                    } // silently ignore if this wasn't in the list when we removed it
                }

                FactionReplace(faction_ref, faction) => {
                    if let Some(old_faction) = self.factions.replace(faction_ref, faction.clone()) {
                        return_node.push_back(FactionReplace(faction_ref.clone(), old_faction));
                    } // silently ignore if no replacement was possible?
                }
            }
        }
        // fill return node
        Ok(return_node)
    }

    pub fn is_empty(&self) -> bool {
        self.districts.is_empty() && self.persons.is_empty() && self.factions.is_empty()
    }

    // todo: precalc and cache this?
    pub fn persons_display_table(&self) -> DisplayTable {
        DisplayTable::from(&self.persons)
    }

    pub fn set_loaded_from(&mut self, maybe_file: Option<PathBuf>) {
        self.loaded_from = maybe_file;
    }

    pub fn get_loaded_from(&self) -> Option<PathBuf> {
        self.loaded_from.clone()
    }

    // todo: precalc and cache this?
    pub fn districts_display_table(&self) -> DisplayTable {
        DisplayTable::from(&self.districts)
    }

    // todo: precalc and cache this?
    pub fn factions_display_table(&self) -> DisplayTable {
        DisplayTable::from(&self.factions)
    }

    pub fn person_list(&self) -> &ManagedList<Person> {
        &self.persons
    }

    pub fn district_list(&self) -> &ManagedList<District> {
        &self.districts
    }

    pub fn faction_list(&self) -> &ManagedList<Faction> {
        &self.factions
    }

    pub fn persons_names(&self) -> Vec<String> {
        self.persons.names_sorted()
    }

    pub fn districts_names(&self) -> Vec<String> {
        self.districts.names_sorted()
    }

    pub fn factions_names(&self) -> Vec<String> {
        self.factions.names_sorted()
    }

    pub fn set_persons_sort(&mut self, index: usize) {
        self.persons.set_sorting(index);
    }

    pub fn set_districts_sort(&mut self, index: usize) {
        self.districts.set_sorting(index);
    }

    pub fn set_factions_sort(&mut self, index: usize) {
        self.factions.set_sorting(index);
    }

    pub fn find_district(&self, name: &str) -> Option<DistrictRef> {
        self.districts.find(name)
    }

    pub fn clone_district(&self, index: &DistrictRef) -> Option<District> {
        self.districts.fetch(index).cloned()
    }

    pub fn find_person(&self, name: &str) -> Option<PersonRef> {
        self.persons.find(name)
    }

    pub fn clone_person(&self, index: &PersonRef) -> Option<Person> {
        self.persons.fetch(index).cloned()
    }

    pub fn find_faction(&self, name: &str) -> Option<FactionRef> {
        self.factions.find(name)
    }

    pub fn clone_faction(&self, index: &FactionRef) -> Option<Faction> {
        self.factions.fetch(index).cloned()
    }

    pub fn view_size(&self, view: MainView) -> usize {
        match view {
            MainView::Factions => self.factions.len(),
            MainView::Persons => self.persons.len(),
            MainView::Districts => self.districts.len(),
        }
    }

    pub fn data_index_valid(&self, index: DataIndex) -> bool {
        match index {
            DataIndex::Nothing => false,
            DataIndex::DistrictIndex(i) => self.district_list().fetch_with_index(i).is_some(),
            DataIndex::PersonIndex(i) => self.person_list().fetch_with_index(i).is_some(),
            DataIndex::FactionIndex(i) => self.faction_list().fetch_with_index(i).is_some(),
        }
    }

    /// This saves all data to a save file
    pub fn save_to_file(&mut self, file_path: &Path) -> Result<()> {
        save_data_to_file(file_path, self)
    }

    /// This exports all data to a JSON file
    pub fn export_to_file(&self, file_path: &Path) -> Result<()> {
        let save_data: SaveData2 = self.into();
        if !save_data.validate() {
            error!(
                "unable to validate data to export ({}), version: {}",
                file_path.to_string_lossy(),
                save_data.save_version
            );
            return Err(anyhow!(
                "unable to validate export data ({}), version: {}",
                file_path.to_string_lossy(),
                save_data.save_version
            ));
        }
        save_to_json(&file_path.with_extension(JSON_EXTENSION), &save_data)
    }

    /// This adds the loaded data to the current data
    pub fn import_from_file(&mut self, file_path: &Path) -> Result<()> {
        let import_data: SaveData2 = {
            // try 2
            match load_from_json::<SaveData2>(&file_path.with_extension(JSON_EXTENSION)) {
                Result::Ok(save2) => save2,
                Err(_) => {
                    // try 1
                    let save1: SaveData1 =
                        load_from_json(&file_path.with_extension(JSON_EXTENSION))?;
                    save1.into()
                }
            }
        };

        debug!(
            "imported {} people, {} districts, {} factions",
            import_data.persons.len(),
            import_data.districts.len(),
            import_data.factions.len()
        );
        if !import_data.validate() {
            error!(
                "unable to validate imported data ({}), version: {}",
                file_path.to_string_lossy(),
                import_data.save_version
            );
            return Err(anyhow!(
                "unable to validate imported data ({}), version: {}",
                file_path.to_string_lossy(),
                import_data.save_version
            ));
        }
        self.load_data(import_data)
    }

    /// This creates a new AppData after loading data from the file path
    pub fn load_from_file(file_path: &Path) -> Result<AppData> {
        save_data_from_file(file_path) // .with_extension(DATA_EXTENSION)
    }

    fn load_data(&mut self, save_data: impl Into<SaveData2>) -> Result<()> {
        // !! Not using return??
        let save_data: SaveData2 = save_data.into();
        let mut post_districts: Vec<(String, Vec<String>)> = Vec::new();
        let mut district_add = save_data
            .districts
            .into_iter()
            .map(|d| {
                let (district, notable) = self.district_from_store(d);
                post_districts.push((district.name().to_string(), notable));
                Action::DistrictAdd(district)
            })
            .collect();

        if let Err(err) = self.do_action(&mut district_add) {
            error!("unable to add districts: {err}");
        }

        let mut person_add = save_data
            .persons
            .into_iter()
            .map(|p| {
                let person = self.person_from_store(p);
                Action::PersonAdd(person)
            })
            .collect();

        if let Err(err) = self.do_action(&mut person_add) {
            error!("unable to add persons: {err}");
        }

        // do district references to persons; no undo
        let mut district_replace: ActionNode = post_districts.into_iter().filter_map(|(district_name, notable)| {
            if let Some(dist_ref) = self.districts.find(&district_name) {
                if let Some(dist) = self.districts.fetch(&dist_ref) {
                    let notables: Vec<PersonRef> = notable.into_iter().filter_map(|p| {
                        let p_ref = self.persons.find(&p);
                        if p_ref.is_none() { error!("unable to find person {p} as notable when loading district {district_name}"); }
                        p_ref
                    }).collect();
                    if !notables.is_empty() {
                        let mut district = dist.clone();
                        district.set_notable(notables);
                        Some(Action::DistrictReplace(dist_ref, district))
                    } else { None }

                } else { None }
            } else { None }
        }).collect();

        if let Err(err) = self.do_action(&mut district_replace) {
            error!("unable to replace districts with notables: {err}");
        }

        let mut post_factions: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
        let mut faction_add = save_data
            .factions
            .into_iter()
            .map(|f| {
                let (faction, allies, enemies) = self.faction_from_store(f);
                post_factions.push((faction.name().to_string(), allies, enemies));
                Action::FactionAdd(faction)
            })
            .collect();

        if let Err(err) = self.do_action(&mut faction_add) {
            error!("unable to add factions: {err}");
        }

        // do faction references to factions
        // note these do not have undo
        let mut faction_replace: ActionNode = post_factions.into_iter().filter_map(|(faction_name, allies, enemies)| {
            if let Some(fac_ref) = self.factions.find(&faction_name) {
                if let Some(fac) = self.factions.fetch(&fac_ref) {
                    let allies: Vec<FactionRef> = allies.into_iter().filter_map(|f| {
                        let f_ref = self.factions.find(&f);
                        if f_ref.is_none() { error!("unable to find faction {f} as ally when loading faction {faction_name}"); }
                        f_ref
                    }).collect();
                    let enemies: Vec<FactionRef> = enemies.into_iter().filter_map(|f| {
                        let f_ref = self.factions.find(&f);
                        if f_ref.is_none() { error!("unable to find faction {f} as enemy when loading faction {faction_name}"); }
                        f_ref
                    }).collect();
                    if !allies.is_empty() || !enemies.is_empty() {
                        let mut faction = fac.clone();
                        if !allies.is_empty() { faction.set_allies(allies); }
                        if !enemies.is_empty() { faction.set_enemies(enemies); }
                        Some(Action::FactionReplace(fac_ref, faction))
                    } else { None }
                } else { None }
            } else { None }
        }).collect();

        if let Err(err) = self.do_action(&mut faction_replace) {
            error!("unable to replace factions with allies and enemies: {err}");
        }

        Ok(())
    }

    // todo: actually we'd want to add this to the current data, and use the file name (_file_path: &Path)
    pub fn test_import_from_json(&mut self) -> Result<()> {
        let data = include_str!("../test_data/test1.json");

        let import: SaveData2 = {
            // try 2
            match serde_json::from_str::<SaveData2>(data) {
                Result::Ok(save2) => save2,
                Err(_) => {
                    // try 1
                    let save1: SaveData1 = serde_json::from_str(data)?;
                    save1.into()
                }
            }
        };

        debug!(
            "imported {} people, {} districts, {} factions",
            import.persons.len(),
            import.districts.len(),
            import.factions.len()
        );
        self.load_data(import)
    }

    fn person_from_store(&self, p_store: PersonStore2) -> Person {
        let mut person: Person = (&p_store).into();
        // todo: convert references
        if let Some(found_in) = p_store.found_in {
            let found_in_ref = self.districts.find(&found_in);
            if found_in_ref.is_none() {
                error!(
                    "unable to find district {} as found_in when loading person {}",
                    found_in,
                    person.name()
                );
            }
            person.set_found_in(found_in_ref);
        }

        person
    }

    /// Returns list of notable Person Strings
    fn district_from_store(&self, d_store: DistrictStore) -> (District, Vec<String>) {
        let district: District = (&d_store).into();
        // todo: convert references

        (district, d_store.notable)
    }

    fn faction_from_store(&self, f_store: FactionStore) -> (Faction, Vec<String>, Vec<String>) {
        let mut faction: Faction = (&f_store).into();
        // todo: convert references
        // hq (option district)
        if let Some(hq) = f_store.hq {
            let hq_ref = self.districts.find(&hq);
            if hq_ref.is_none() {
                error!(
                    "unable to find district {} as hq when loading faction {}",
                    hq,
                    faction.name()
                );
            }
            faction.set_hq(hq_ref);
        }

        // turf (vec district)
        let turf = f_store
            .turf
            .into_iter()
            .filter_map(|d| {
                let d_ref = self.districts.find(&d);
                if d_ref.is_none() {
                    error!(
                        "unable to find district {} as turf when loading faction {}",
                        d,
                        faction.name()
                    );
                }
                d_ref
            })
            .collect();
        faction.set_turf(turf);

        // leader (option person)
        if let Some(leader) = f_store.leader {
            let leader_ref = self.persons.find(&leader);
            if leader_ref.is_none() {
                error!(
                    "unable to find person {} as leader when loading faction {}",
                    leader,
                    faction.name()
                );
            }
            faction.set_leader(leader_ref);
        }

        // notable (vec person)
        let notable = f_store
            .notable
            .into_iter()
            .filter_map(|p| {
                let p_ref = self.persons.find(&p);
                if p_ref.is_none() {
                    error!(
                        "unable to find person {} as notable when loading faction {}",
                        p,
                        faction.name()
                    );
                }
                p_ref
            })
            .collect();
        faction.set_notable(notable);

        // allies (vec faction)
        // enemies (vec faction)

        (faction, f_store.allies, f_store.enemies)
    }
}

fn save_data_to_file(file_path: &Path, data: &AppData) -> Result<()> {
    let save_data: SaveData2 = data.into();
    if save_data.validate() {
        let buffer = pot::to_vec::<SaveData2>(&save_data)?;
        save_to_save(file_path, save_data.save_version, buffer)
    } else {
        Err(anyhow!("unable to validate save data - not saved"))
    }
}

fn save_data_from_file(file_path: &Path) -> Result<AppData> {
    let data = match load_from_save(file_path) {
        Result::Ok((save_version, buffer)) => match save_version {
            SAVE2_VERSION => pot::from_reader::<SaveData2, _>(buffer)?,
            SAVE1_VERSION => pot::from_reader::<SaveData1, _>(buffer)?.into(),
            _ => {
                error!("invalid save file version {save_version}");
                return Err(anyhow!("invalid save file version {save_version}"));
            }
        },

        Err(e) => {
            error!("unable to read save file header: {e}");
            return Err(e);
        }
    };

    if data.validate() {
        // todo: should we validate the loaded savedata, rather than the data to convert?
        let mut ret: AppData = data.into();
        ret.loaded_from = Some(file_path.to_path_buf());
        Ok(ret)
    } else {
        error!("unable to validate loaded save data");
        Err(anyhow!("unable to validate save data"))
    }
}

// Save data
const SAVE_SCHEMA: &str = "BladesFactionsData";

// TODO: can we have one struct that remains consistent, and has the conversion to app data
// and a set of save data formats that do the loading?
//

// ====================
// SaveData2
const SAVE2_VERSION: u16 = 2;

#[derive(Debug, Serialize, Deserialize)]
struct SaveData2 {
    save_schema: String,
    save_version: u16,
    persons: Vec<PersonStore2>,
    districts: Vec<DistrictStore>,
    factions: Vec<FactionStore>,
}

impl SaveData2 {
    fn validate(&self) -> bool {
        self.save_schema == SAVE_SCHEMA && self.save_version == SAVE2_VERSION
    }
}

impl From<SaveData2> for AppData {
    fn from(save_data: SaveData2) -> Self {
        let mut app_data = AppData::default();
        if let Err(e) = app_data.load_data(save_data) {
            error!("unable to load save version 2 data: {e}");
        }
        app_data
    }
}

impl From<&AppData> for SaveData2 {
    fn from(input_data: &AppData) -> Self {
        SaveData2 {
            save_schema: SAVE_SCHEMA.to_string(),
            save_version: SAVE2_VERSION,
            persons: input_data.persons.borrow().into(),
            districts: input_data.districts.borrow().into(),
            factions: input_data.factions.borrow().into(),
        }
    }
}

impl From<SaveData1> for SaveData2 {
    fn from(save_data1: SaveData1) -> Self {
        assert!(save_data1.validate()); // This is too extreme, but we want to check the version and scheme before loading the data
        SaveData2 {
            save_schema: save_data1.save_schema,
            save_version: save_data1.save_version,
            persons: save_data1.persons.into_iter().map(|p| p.into()).collect(),
            districts: save_data1.districts,
            factions: save_data1.factions,
        }
    }
}

// ====================
// SaveData1
const SAVE1_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct SaveData1 {
    save_schema: String,
    save_version: u16,
    persons: Vec<PersonStore1>,
    districts: Vec<DistrictStore>,
    factions: Vec<FactionStore>,
}

impl SaveData1 {
    fn validate(&self) -> bool {
        self.save_schema == SAVE_SCHEMA && self.save_version == SAVE1_VERSION
    }
}

impl From<SaveData1> for AppData {
    fn from(save_data: SaveData1) -> Self {
        let mut app_data = AppData::default();
        if let Err(e) = app_data.load_data(save_data) {
            error!("unable to load save version 1 data: {e}");
        }
        app_data
    }
}

impl From<&AppData> for SaveData1 {
    fn from(input_data: &AppData) -> Self {
        SaveData1 {
            save_schema: SAVE_SCHEMA.to_string(),
            save_version: SAVE1_VERSION,
            persons: input_data.persons.borrow().into(),
            districts: input_data.districts.borrow().into(),
            factions: input_data.factions.borrow().into(),
        }
    }
}

// ----------------------------------------

// Is it true that while the index can change, the relative order will remain the same?
// !! Assuming: even if index changed, relative order will remain the same
// BUT you cannot assume the indexed item is valid - you must check it before using it
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataIndex {
    Nothing,
    DistrictIndex(usize),
    PersonIndex(usize),
    FactionIndex(usize),
}

impl DataIndex {
    pub fn index(&self) -> Option<usize> {
        match self {
            DataIndex::DistrictIndex(i)
            | DataIndex::FactionIndex(i)
            | DataIndex::PersonIndex(i) => Some(*i),

            _ => None,
        }
    }
}
