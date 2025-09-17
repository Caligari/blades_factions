use std::collections::BTreeMap;

use crate::app_data::DataIndex;



#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorPair (DataIndex, DataIndex);

#[allow(dead_code)]
impl ActorPair {
    pub fn new ( actor1: DataIndex, actor2: DataIndex ) -> ActorPair {
        assert_ne!(actor1, actor2);
        assert_ne!(actor1, DataIndex::Nothing);
        assert_ne!(actor2, DataIndex::Nothing);

        if actor1 < actor2 {
            ActorPair(actor1, actor2)
        } else {
            ActorPair(actor2, actor1)
        }
    }
}

#[allow(dead_code)]
pub struct Relationships {
    table: BTreeMap<ActorPair, Attitude>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Relationship {
    actors: ActorPair,
    attitude: Attitude,
}



#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Attitude {
    Minus3,
    Minus2,
    Minus1,
    #[default]
    Zero,
    Plus1,
    Plus2,
    Plus3,
}
