use bitvec::array::BitArray;

pub trait Component {}

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    mask: BitArray,
}

pub struct Archetype {
    
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}