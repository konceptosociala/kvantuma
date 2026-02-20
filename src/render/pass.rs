use crate::render::material::Material;

use super::*;

pub struct ComputePass<'a> {
    pub(super) pass: wgpu::ComputePass<'a>,
}

pub struct ComputeDescriptor<'a, 'b, T> {
    pub instance_data: Option<&'b dyn InstanceData<UniformData = T>>,
    pub pipeline: &'a Pipeline,
    pub shader_resources: &'b [&'a ShaderResource],
    pub size: UVec2,
}

impl<'a> ComputePass<'a> {
    pub fn compute<T: Pod>(&mut self, descriptor: ComputeDescriptor<'a, '_, T>) {
        if let Pipeline::Compute(p) = descriptor.pipeline {
            self.pass.set_pipeline(p);
        } else {
            panic!("Cannot use render pipeline in compute() command");
        }

        for (i, binding) in descriptor.shader_resources.iter().enumerate() {
            self.pass.set_bind_group(i as u32, &binding.bind_group, &[]);
        }

        if let Some(instance_data) = descriptor.instance_data {
            self.pass.set_push_constants(
                0,
                bytemuck::cast_slice(&[instance_data.uniform_data()]),
            );
        }

        self.pass.dispatch_workgroups(
            descriptor.size.x / 16, 
            descriptor.size.y / 16, 
            1,
        );
    }
}

/// Represents a render pass used for drawing.
pub struct RenderPass<'a> {
    pub(super) pass: wgpu::RenderPass<'a>
}

pub struct DrawDescriptor<'a, 'b, T, M> {
    pub drawable: Option<&'b dyn Drawable>,
    pub instance_data: Option<&'b dyn InstanceData<UniformData = T>>,
    pub material: &'a M,
    pub shader_resources: &'b [&'a ShaderResource],
}

// impl<'a> RenderPass<'a> {
//     pub fn draw<T: Pod, M: Material>(
//         &mut self,
//         render_world: &RenderRegistry,
//         descriptor: DrawDescriptor<'a, '_, T>,
//     ) {
//         if let Pipeline::Render(p) = descriptor.pipeline {
//             self.pass.set_pipeline(p);
//         } else {
//             panic!("Cannot use compute pipeline in draw() command");
//         }

//         for (i, binding) in descriptor.shader_resources.iter().enumerate() {
//             self.pass.set_bind_group(i as u32, &binding.bind_group, &[]);
//         }

//         if let Some(instance_data) = descriptor.instance_data {
//             self.pass.set_push_constants(
//                 wgpu::ShaderStages::VERTEX_FRAGMENT,
//                 0,
//                 bytemuck::cast_slice(&[instance_data.uniform_data()]),
//             );
//         }
        
//         if let Some(drawable) = descriptor.drawable {
//             self.pass.set_vertex_buffer(0, render_world.get_buffer(drawable.vertex_buffer()).unwrap().inner().slice(..)); 
//             self.pass.draw(0..render_world.get_buffer(drawable.vertex_buffer()).unwrap().capacity() as u32, 0..1);
//         } else {
//             self.pass.draw(0..6, 0..1);
//         }
//     }
// }