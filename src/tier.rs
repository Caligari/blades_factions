use std::fmt::Display;

use serde::{Deserialize, Serialize};





#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
}

impl Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Tier::*;

        write!(f, "{}",
            match self {
                Tier0 => "0",
                Tier1 => "I",
                Tier2 => "II",
                Tier3 => "III",
                Tier4 => "IV",
                Tier5 => "V",
            }
        )
    }
}