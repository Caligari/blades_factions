use serde::{Deserialize, Serialize};





#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
}
