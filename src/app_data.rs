use std::path::Path;

use anyhow::{Result, Ok, anyhow};
use serde::{Deserialize, Serialize};

use crate::app::load_from_pot;

const DATA_EXTENSION: &str = "pot";


// Default should be empty
#[derive(Debug, Default)]
pub struct AppData {

}

#[allow(dead_code)]
impl AppData {
    pub fn save_to_file ( &self ) {

    }

    pub fn load_from_file ( file_path: &Path ) -> Result<AppData> {
        load_save_data(&file_path.with_extension(DATA_EXTENSION))
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
