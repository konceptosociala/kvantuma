use bitvec::array::BitArray;

pub trait Component {}

#[derive(Default)]
pub struct World {
    _archetypes: Vec<Archetype>,
    _mask: BitArray,
}

pub struct Archetype {
    
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}