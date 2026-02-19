use crate::ecs::world::World;

pub trait System {
    fn execute(&self, world: &mut World);
}