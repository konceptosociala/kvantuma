use glam::{Vec2, Vec3};
use kvantuma::{
    app::{
        App, Game,
        window::{WindowDescriptor, WindowMode},
    }, 
    ecs::world::World, 
    render::{
        Drawable, RenderDevice, RenderSurface, 
        buffer::BufferHandle, 
        error::RenderError, 
        material::{TintedTextureMaterial, Vertex}, 
        pass::DrawDescriptor, 
        registry::RenderRegistry,
        types::*,
    }
};

pub struct Triangle {
    pub vertex_data: [Vertex; 3],
    pub vertex_buffer: Option<BufferHandle>,
}

impl Drawable for Triangle {
    fn update(
        &mut self, 
        render_device: &mut RenderDevice,
        world: &mut RenderRegistry,
    ) {
        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(
                world.new_buffer::<Vertex>(render_device, 3, BufferUsages::VERTEX)
            );
        }

        let Some(handle) = self.vertex_buffer else { unreachable!() };
        
        world
            .get_buffer(handle) 
            .expect("Cannot call update() on Triangle")
            .fill_exact(render_device, 0, &self.vertex_data)
            .unwrap();
    }
    

    fn vertex_buffer(&self) -> BufferHandle {
        self.vertex_buffer
            .expect("Triangle is not set up with update()")
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertex_data: [
                Vertex {
                    position: Vec3::new(0.0, 0.5, 0.0),
                    texcoord: Vec2::new(0.5, 0.0),
                },
                Vertex {
                    position: Vec3::new(-0.5, -0.5, 0.0),
                    texcoord: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: Vec3::new(0.5, -0.5, 0.0),
                    texcoord: Vec2::new(1.0, 1.0),
                },
            ],
            vertex_buffer: None,
        }
    }
}

struct KvantumaGame {
    // materials: MaterialRegistry,
    registry: RenderRegistry,
    material: Option<TintedTextureMaterial>,
    triangle: Option<Triangle>,
}

impl Game for KvantumaGame {
    fn init(&mut self, world: &mut World, render_device: &mut RenderDevice) -> anyhow::Result<()> {
        self.registry.register_material::<TintedTextureMaterial>(render_device);
        
        self.triangle = Some(Triangle::default());
        self.triangle.as_mut().unwrap().update(render_device, &mut self.registry);

        self.material = Some(TintedTextureMaterial::new(
            "assets/textures/test.png", 
            Vec3::new(0.0, 1.0, 0.5), 
            render_device, 
            &mut self.registry,
        )?);

        Ok(())
    }

    fn update(&mut self, _world: &mut World) -> anyhow::Result<()> {
        Ok(())
    }

    fn input(&mut self, _event: &glfw::WindowEvent, _world: &mut World) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(&mut self, _world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        {
            let mut render_pass = ctx.render_pass(canvases, render_device.depth_texture());

            render_pass.draw(render_device, &self.registry, DrawDescriptor::<(), _> {
                drawable: Some(self.triangle.as_ref().unwrap()),
                instance_data: None,
                material: self.material.as_ref().unwrap(),
            });
        }

        ctx.apply(canvas, render_device);

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    App::new(
        WindowDescriptor {
            width: 1280,
            height: 720,
            title: "KVΛNTUMA",
            mode: WindowMode::Windowed,
        }, 
        KvantumaGame {
            registry: RenderRegistry::new(),
            material: None,
            triangle: None,
        },
    )?.run();

    Ok(())
}
