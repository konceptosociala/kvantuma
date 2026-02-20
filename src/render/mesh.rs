use crate::render::Drawable;

pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>
}

impl<V> Drawable for Mesh<V> {
    fn update(&mut self, render_device: &mut super::RenderDevice, world: &mut super::registry::RenderRegistry) {
        todo!()
    }

    fn vertex_buffer(&self) -> super::buffer::BufferHandle {
        todo!()
    }
}