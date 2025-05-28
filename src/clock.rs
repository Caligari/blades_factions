use serde::{Deserialize, Serialize};




#[allow(dead_code)]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Clock {
    name: String,
    description: String,  // ?
    parts: u8,
    ticked: u8,
}

#[allow(dead_code)]
impl Clock {
    pub fn new_clock_4 ( name: &str, description: &str ) -> Self {
        Self::new(4, name, description)
    }

    pub fn new_clock_6 ( name: &str, description: &str ) -> Self {
        Self::new(6, name, description)
    }

    pub fn new_clock_8 ( name: &str, description: &str ) -> Self {
        Self::new(8, name, description)
    }

    pub fn new_clock_12 ( name: &str, description: &str ) -> Self {
        Self::new(12, name, description)
    }

    fn new ( size: u8, name: &str, description: &str ) -> Self {
        Clock {
            name: name.to_string(),
            description: description.to_string(),
            parts: size,
            ticked: 0,
        }
    }

    pub fn name ( &self ) -> &str {
        &self.name
    }

    pub fn description ( &self ) -> &str {
        &self.description
    }

    /// Returns the number of ticks and the total number of parts
    pub fn status ( &self ) -> (u8, u8) {
        (self.ticked, self.parts)
    }

    /// Increases the number of ticks by 1, and returns whether it is finished now
    pub fn tick ( &mut self ) -> bool {
        self.ticked = (self.ticked + 1).min(self.parts);
        self.ticked >= self.parts
    }
}
