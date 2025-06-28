use std::fmt::Display;

use serde::{Deserialize, Serialize};




#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dots {
    #[default]
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Display for Dots {
    fn fmt ( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
        use Dots::*;

        write!(f, "{}",
            match self {
                Zero => "○".to_string(),
                One => "●".to_string(),
                Two => "●●".to_string(),
                Three => "●●●".to_string(),
                Four => "●●●●".to_string(),
                Five => "●●●●●".to_string(),
            }
        )
    }
}
