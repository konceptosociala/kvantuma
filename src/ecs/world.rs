use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec3;
use slotmap::Key;

use super::archetype::*;
use super::component::*;
use crate::component;
use crate::render::buffer::BufferHandle;
use crate::render::material::TintedTextureMaterial;
use crate::render::texture::TextureHandle;

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    component_meta: Vec<ComponentMeta>,
    next_entity: EntityId,
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}

#[derive(Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
struct PodType { a: i32 }

component! { POD: PodType }
component! { EXTERN: TintedTextureMaterial }

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

use glfw::Glfw;

component! { EXTERN: Glfw }

#[derive(Debug)]
pub struct ErasedQueryResult<'a> {
    pub entity: EntityId,
    pub components: Vec<&'a [u8]>,
}

impl World {
    pub fn query_erased(&mut self, component_ids: &[ComponentId]) -> Vec<ErasedQueryResult<'_>> {
        let mut results = Vec::new();

        for archetype in &mut self.archetypes {
            if archetype.has_components(component_ids) {
                let len = archetype.entities.len();

                let column_indices: Vec<usize> = component_ids
                    .iter()
                    .map(|&id| archetype.columns.iter().position(|col| col.meta.id == id).unwrap())
                    .collect();
                let columns: Vec<&Column> = column_indices
                    .iter()
                    .map(|&idx| &archetype.columns[idx])
                    .collect();

                for i in 0..len {
                    let mut comps = Vec::with_capacity(component_ids.len());
                    for col in &columns {
                        unsafe {
                            let ptr = col.ptr.as_ptr().add(i * col.meta.layout.size());
                            let slice = std::slice::from_raw_parts(ptr, col.meta.layout.size());
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

    pub fn query_two_test<A: Component, B: Component>(&mut self) -> (&A, &B) {
        let result = self.query_erased(&[A::component_id(), B::component_id()]);
        let first = result.first().expect("Should have at least one result");
        let a_bytes = first.components[0];
        let b_bytes = first.components[1];
        let a = unsafe { &*(a_bytes.as_ptr() as *const A) };
        let b = unsafe { &*(b_bytes.as_ptr() as *const B) };
        (a, b)
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

pub fn x() {
    let mut world = World::new();
    world.spawn((
        // PodType { a: 1332 },
        glfw::init_no_callbacks().unwrap(),
        TintedTextureMaterial {
            albedo: TextureHandle::null(),
            tint_buffer: BufferHandle::null(),
            tint: Vec3::NEG_Z,
        }
    ));


    let result = world.query_erased(&[
        Glfw::component_id(),
        TintedTextureMaterial::component_id(),
    ]);

    dbg!(&result);

    let res2 = world.query_two_test::<Glfw, TintedTextureMaterial>();

    dbg!(res2);
}