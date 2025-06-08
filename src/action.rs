use std::collections::VecDeque;

use crate::{district::District, faction::Faction, managed_list::{DistrictRef, FactionRef, PersonRef}, person::Person};


pub type ActionNode = VecDeque<Action>;

#[allow(dead_code)]
#[derive(Clone)]
pub enum Action {
    // Add district
    DistrictAdd (District),
    // Remove district
    DistrictRemove (DistrictRef),
    // Replace district
    DistrictReplace (DistrictRef, District),

    // Add person
    PersonAdd (Person),
    // Remove person
    PersonRemove (PersonRef),
    // Replace person
    PersonReplace (PersonRef, Person),

    // Add faction
    FactionAdd (Faction),
    // Remove faction
    FactionRemove (FactionRef),
    // Replace faction
    FactionReplace (FactionRef, Faction),

    // Clear all
    // ClearAll,
}
