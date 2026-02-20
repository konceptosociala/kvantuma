use std::{alloc::{Layout, alloc}, ptr::NonNull};

use crate::ecs::component::{Component, ComponentId, ComponentKind, ErasedComponent};

use super::component::ComponentMeta;

const MAX_COMPONENTS: usize = 256;
const WORDS: usize = MAX_COMPONENTS / u64::BITS as usize;

pub type EntityId = u32;

#[derive(Clone)]
pub struct ArchetypeMask {
    words: [u64; WORDS],
}

impl ArchetypeMask {
    pub fn from_ids(ids: &[ComponentId]) -> Self {
        if ids.len() > MAX_COMPONENTS {
            panic!("Too many components in archetype {}", ids.len());
        }

        let mut mask = ArchetypeMask { words: [0; WORDS] };
        for &id in ids {
            let word_index = (id as usize) / u64::BITS as usize;
            let bit_index = (id as usize) % u64::BITS as usize;
            mask.words[word_index] |= 1 << bit_index;
        }

        mask
    }

    pub fn contains(&self, other: &ArchetypeMask) -> bool {
        self.words.iter().zip(other.words.iter())
            .all(|(a, b)| (a & b) == *b)
    }
}

pub struct Column {
    pub ptr: NonNull<u8>,
    pub len: usize,
    pub capacity: usize,
    pub meta: ComponentMeta,
}

impl Column {
    pub fn new(
        capacity: usize, 
        id: u32, 
        layout: Layout,
        kind: ComponentKind,
        drop_fn: Option<unsafe fn(*mut u8)>,
    ) -> Column {
        let total_size = layout.size() * capacity;
        let ptr = unsafe { alloc(Layout::from_size_align(total_size, layout.align()).unwrap()) };
        let meta = ComponentMeta {
            id,
            kind,
            layout,
            drop_fn,
        };

        Column {
            ptr: NonNull::new(ptr).unwrap(),
            len: 0,
            capacity,
            meta,
        }
    }

    pub fn push(&mut self, component: &dyn Component) {
        if self.len >= self.capacity {
            let new_capacity = self.capacity * 2;
            let new_size = self.meta.layout.size() * new_capacity;
            let new_ptr = unsafe { alloc(Layout::from_size_align(new_size, self.meta.layout.align()).unwrap()) };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ptr.as_ptr(),
                    new_ptr,
                    self.meta.layout.size() * self.len,
                );
                std::alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align(self.meta.layout.size() * self.capacity, self.meta.layout.align()).unwrap()
                );
            }
            self.ptr = NonNull::new(new_ptr).unwrap();
            self.capacity = new_capacity;
        }

        let offset = self.len * self.meta.layout.size();
        unsafe {
            std::ptr::copy_nonoverlapping(
                component as *const _ as *const u8,
                self.ptr.as_ptr().add(offset),
                self.meta.layout.size(),
            );
        }
        self.len += 1;
    }

    pub fn push_erased(&mut self, component: &ErasedComponent) {
        if self.len >= self.capacity {
            let new_capacity = self.capacity * 2;
            let new_size = self.meta.layout.size() * new_capacity;
            let new_ptr = unsafe { alloc(Layout::from_size_align(new_size, self.meta.layout.align()).unwrap()) };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ptr.as_ptr(),
                    new_ptr,
                    self.meta.layout.size() * self.len,
                );
                std::alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align(self.meta.layout.size() * self.capacity, self.meta.layout.align()).unwrap()
                );
            }
            self.ptr = NonNull::new(new_ptr).unwrap();
            self.capacity = new_capacity;
        }

        let offset = self.len * self.meta.layout.size();
        unsafe {
            std::ptr::copy_nonoverlapping(
                component.data,
                self.ptr.as_ptr().add(offset),
                self.meta.layout.size(),
            );
        }
        self.len += 1;
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        for i in 0..self.len {
            if let Some(drop_fn) = self.meta.drop_fn {
                unsafe {
                    drop_fn(self.ptr.as_ptr().add(i * self.meta.layout.size()));
                }
            }
        }
        unsafe {
            std::alloc::dealloc(self.ptr.as_ptr(), Layout::from_size_align(self.meta.layout.size() * self.capacity, self.meta.layout.align()).unwrap());
        }
    }
}

pub struct Archetype {
    pub mask: ArchetypeMask,
    pub columns: Vec<Column>,
    pub entities: Vec<EntityId>,
}

impl Archetype {
    pub fn new(
        mask: ArchetypeMask,
        columns: Vec<Column>,
    ) -> Archetype {
        Archetype {
            mask,
            columns,
            entities: vec![],
        }
    }

    pub fn has_components(&self, ids: &[ComponentId]) -> bool {
        let mask = ArchetypeMask::from_ids(ids);
        self.mask.contains(&mask)
    }

    pub fn get_column_with_component(&mut self, id: ComponentId) -> Option<&Column> {
        self.columns
            .iter()
            .find(|col| col.meta.id == id)
    }

    pub fn get_column_with_component_mut(&mut self, id: ComponentId) -> Option<&mut Column> {
        self.columns
            .iter_mut()
            .find(|col| col.meta.id == id)
    }

    pub fn add_entity(&mut self, id: EntityId) {
        self.entities.push(id);
    }
}