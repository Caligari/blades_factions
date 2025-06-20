use std::path::Path;

use anyhow::{Result, Ok, anyhow};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{action::{Action, ActionNode}, app::load_from_pot, app_display::DisplayTable, district::District, faction::{Faction, FactionStore}, managed_list::{FactionRef, ManagedList, Named}, person::Person};

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
        DisplayTable::from(&self.persons)
    }

    // todo: precalc and cache this?
    pub fn districts_display_table ( &self ) -> DisplayTable {
        DisplayTable::from(&self.districts)
    }

    // todo: precalc and cache this?
    pub fn factions_display_table ( &self ) -> DisplayTable {
        DisplayTable::from(&self.factions)
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

        if let Err(err) = self.do_action(&district_add) {
            error!("unable to add districts: {}", err);
        }

        if let Err(err) = self.do_action(&person_add) {
            error!("unable to add persons: {}", err);
        }

        let mut post_factions: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
        let faction_add = import.factions.into_iter().map(|f| {
            let (faction, allies, enemies) = self.faction_from_store(f);
            post_factions.push((faction.name().to_string(), allies, enemies));
            Action::FactionAdd(faction)
        }).collect();

        if let Err(err) = self.do_action(&faction_add) {
            error!("unable to add factions: {}", err);
        }

        // do faction references to factions
        // note these do not have undo
        let faction_replace: ActionNode = post_factions.into_iter().filter_map(|(faction_name, allies, enemies)| {
            if let Some(fac_ref) = self.factions.find(&faction_name) {
                if let Some(fac) = self.factions.fetch(&fac_ref) {
                    let allies: Vec<FactionRef> = allies.into_iter().filter_map(|f| {
                        let f_ref = self.factions.find(&f);
                        if f_ref.is_none() { error!("unable to find faction {} as ally when loading faction {}", f, faction_name); }
                        f_ref
                    }).collect();
                    let enemies: Vec<FactionRef> = enemies.into_iter().filter_map(|f| {
                        let f_ref = self.factions.find(&f);
                        if f_ref.is_none() { error!("unable to find faction {} as enemy when loading faction {}", f, faction_name); }
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

        if let Err(err) = self.do_action(&faction_replace) {
            error!("unable to replace factions with allies and enemies: {}", err);
        }

        Ok(())
    }

    fn faction_from_store ( &self, f_store: FactionStore )-> (Faction, Vec<String>, Vec<String>) {
        let mut faction: Faction = (&f_store).into();
        // todo: convert references
        // hq (option district)
        if let Some(hq) = f_store.hq {
            let hq_ref = self.districts.find(&hq);
            if hq_ref.is_none() { error!("unable to find district {} as hq when loading faction {}", hq, faction.name()); }
            faction.set_hq(hq_ref);
        }

        // turf (vec district)
        let turf = f_store.turf.into_iter().filter_map(|d| {
            let d_ref = self.districts.find(&d);
            if d_ref.is_none() { error!("unable to find district {} as turf when loading faction {}", d, faction.name()); }
            d_ref
        }).collect();
        faction.set_turf(turf);

        // leader (option person)
        if let Some(leader) = f_store.leader {
            let leader_ref = self.persons.find(&leader);
            if leader_ref.is_none() { error!("unable to find person {} as leader when loading faction {}", leader, faction.name()); }
            faction.set_leader(leader_ref);
        }

        // notable (vec person)
        let notable = f_store.notable.into_iter().filter_map(|p| {
            let p_ref = self.persons.find(&p);
            if p_ref.is_none() { error!("unable to find person {} as notable when loading faction {}", p, faction.name()); }
            p_ref
        }).collect();
        faction.set_notable(notable);

        // allies (vec faction)
        // enemies (vec faction)

        (faction, f_store.allies, f_store.enemies)
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
