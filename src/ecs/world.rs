use super::archetype::*;
use super::component::*;

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    next_entity: EntityId,
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}

impl World {
    pub fn spawn(&mut self, components: impl ComponentsBundle) -> EntityId {
        let mut ids = vec![];

        components.for_each(&mut |comp| {
            let id = comp.id();
            ids.push(id);
        });

        ids.sort();

        if let Some(archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.has_components(&ids)) 
        {
            components.for_each(&mut |comp| {
                let id = comp.id();
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push(comp);
            });

            let id = self.next_entity;
            archetype.add_entity(id);
            self.next_entity += 1;
            
            id
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.for_each(&mut |comp| {
                let mut col = Column::new(64, comp.id(), comp.layout(), comp.kind(), comp.drop_fn());
                col.push(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let id = self.next_entity;
            archetype.add_entity(id);
            self.archetypes.push(archetype);
            self.next_entity += 1;
            
            id
        }
    }

    pub fn spawn_erased(&mut self, components: &[ErasedComponent]) -> EntityId {
        let mut ids = components
            .iter()
            .map(|comp| comp.id)
            .collect::<Vec<_>>();

        ids.sort();

        if let Some(archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.has_components(&ids)) 
        {
            components.iter().for_each(|comp| {
                let id = comp.id;
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push_erased(comp);
            });

            let id = self.next_entity;
            archetype.add_entity(id);
            self.next_entity += 1;
            
            id
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.iter().for_each(|comp| {
                let mut col = Column::new(64, comp.id, comp.layout, comp.kind, comp.drop_fn);
                col.push_erased(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let id = self.next_entity;
            archetype.add_entity(id);
            self.archetypes.push(archetype);
            self.next_entity += 1;
            
            id
        }
    }
}

#[derive(Debug)]
pub struct ErasedQueryResult<'a> {
    pub entity: EntityId,
    pub components: Vec<ComponentQuery<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    Read,
    Write,
}

pub const READ: Access = Access::Read;
pub const WRITE: Access = Access::Write;

#[derive(Debug)]
pub enum ComponentQuery<'a> {
    Read(&'a [u8]),
    Write(&'a mut [u8]),
}

impl World {
    pub fn query_erased(&mut self, components: &[(ComponentId, Access)]) -> Vec<ErasedQueryResult<'_>> {
        let mut results = Vec::new();
        let ids = components
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for archetype in &mut self.archetypes {
            if archetype.has_components(&ids) {
                let len = archetype.entities.len();

                let column_indices: Vec<(usize, Access)> = components
                    .iter()
                    .map(|(id, access)| (
                        archetype.columns.iter().position(|col| col.meta.id == *id).unwrap(),
                        *access
                    ))
                    .collect();
                let columns: Vec<(&Column, Access)> = column_indices
                    .iter()
                    .map(|&(idx, access)| (&archetype.columns[idx], access))
                    .collect();

                for i in 0..len {
                    let mut comps = Vec::with_capacity(components.len());
                    for (col, access) in &columns {
                        unsafe {
                            let ptr = col.ptr.as_ptr().add(i * col.meta.layout.size());
                            let slice = match *access {
                                READ => ComponentQuery::Read(
                                    std::slice::from_raw_parts(ptr, col.meta.layout.size()),
                                ),
                                WRITE => ComponentQuery::Write(
                                    std::slice::from_raw_parts_mut(ptr, col.meta.layout.size()),
                                ),
                            };
                            comps.push(slice);
                        }
                    }
                    results.push(ErasedQueryResult {
                        entity: archetype.entities[i],
                        components: comps,
                    });
                }
            }
        }

        results
    }

    pub fn query<'w, Q: Query<'w>>(&'w mut self) -> Vec<Q::Result> {
        Q::query_world(self)
    }
}

pub trait Query<'w> {
    type Result: 'w;

    fn query_world(world: &'w mut World) -> Vec<Self::Result>;
}

impl<'w, A: Component + 'w> Query<'w> for &A {
    type Result = &'w A;

    fn query_world(world: &'w mut World) -> Vec<Self::Result> {
        world
            .query_erased(&[(A::component_id(), READ)])
            .into_iter()
            .map(|res| {
                let ComponentQuery::Read(comp_a) = &res.components[0] else { unreachable!() };
                unsafe { &*(comp_a.as_ptr() as *const A) }
            })
            .collect()
    }
}

impl<'w, A: Component + 'w, B: Component + 'w> Query<'w> for (&A, &B) {
    type Result = (&'w A, &'w B);

    fn query_world(world: &'w mut World) -> Vec<Self::Result> {
        world
            .query_erased(&[
                (A::component_id(), READ),
                (B::component_id(), READ),
            ])
            .into_iter()
            .map(|res| {
                let ComponentQuery::Read(comp_a) = &res.components[0] else { unreachable!() };
                let ComponentQuery::Read(comp_b) = &res.components[1] else { unreachable!() };
                (
                    unsafe { &*(comp_a.as_ptr() as *const A) },
                    unsafe { &*(comp_b.as_ptr() as *const B) }
                )
            })
            .collect()
    }
}

pub trait ComponentsBundle {
    fn for_each(&self, f: &mut dyn FnMut(&dyn Component));
}

macro_rules! impl_components_bundle_tuple {
    () => {};
    ($($name:ident),+) => {
        impl<$($name: Component),+> ComponentsBundle for ($($name,)+) {
            fn for_each(&self, f: &mut dyn FnMut(&dyn Component)) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(f($name);)+
            }
        }
    };
}

impl_components_bundle_tuple! { A }
impl_components_bundle_tuple! { A, B }
impl_components_bundle_tuple! { A, B, C }
impl_components_bundle_tuple! { A, B, C, D }
impl_components_bundle_tuple! { A, B, C, D, E }
impl_components_bundle_tuple! { A, B, C, D, E, F }
impl_components_bundle_tuple! { A, B, C, D, E, F, G }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K, L }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K, L, M }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K, L, M, N }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P }