use std::{any::TypeId, collections::HashMap};

use bytemuck::Pod;
use image::ImageError;
use slotmap::SlotMap;

use crate::render::pipeline::RenderPipelineDescriptor;
use crate::render::texture::TextureDescriptor;

use super::types::*;
use super::{
    RenderDevice,
    buffer::{BufferHandle, BufferStorage},
    material::Material,
    pipeline::Pipeline,
    texture::{Texture, TextureHandle},
};

#[derive(Default)]
pub struct RenderRegistry {
    pipelines: HashMap<TypeId, Pipeline>,
    buffers: SlotMap<BufferHandle, BufferStorage>,
    textures: SlotMap<TextureHandle, Texture>,
}

impl RenderRegistry {
    pub fn new() -> RenderRegistry {
        RenderRegistry::default()
    }

    pub fn register_material<M: Material + 'static>(&mut self, render_device: &RenderDevice) {
        self.pipelines
            .entry(TypeId::of::<M>())
            .or_insert(
                Pipeline::new_render(render_device, &RenderPipelineDescriptor {
                    shader: M::shader(),
                    bindings: &[&M::shader_resource_layout(render_device)],
                    label: &pretty_type_name::pretty_type_name::<M>(),
                    vertex_layout: M::vertex_layout(),
                    surface_formats: &[render_device.surface_format()],
                })
            );
    }

    pub fn new_buffer<T: Pod>(
        &mut self,
        render_device: &RenderDevice,
        capacity: usize,
        usage: BufferUsages,
    ) -> BufferHandle {
        self.buffers
            .insert(BufferStorage::new::<T>(render_device, capacity, usage))
    }

    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&BufferStorage> {
        self.buffers.get(handle)
    }

    pub fn get_buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut BufferStorage> {
        self.buffers.get_mut(handle)
    }

    pub fn new_texture(
        &mut self,
        render_device: &RenderDevice,
        descriptor: TextureDescriptor,
    ) -> TextureHandle {
        self.textures
            .insert(Texture::new(render_device, descriptor))
    }

    pub fn load_texture(
        &mut self,
        render_device: &RenderDevice,
        path: &str,
        mut descriptor: TextureDescriptor,
    ) -> Result<TextureHandle, ImageError> {
        let image = image::open(path)?
            .to_rgba8();
        descriptor.width = image.width();
        descriptor.height = image.height();

        let texture = Texture::new(render_device, descriptor);
        texture.fill(render_device, image);

        Ok(self.textures.insert(texture))
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn get_texture_mut(&mut self, handle: TextureHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle)
    }
}
