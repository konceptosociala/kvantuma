pub trait Component {}

#[derive(Default)]
pub struct World {

}

impl World {
    pub fn new() -> World {
        World::default()
    }
}