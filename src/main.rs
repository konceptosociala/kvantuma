use kvantuma::{
    app::{
        App, Game,
        window::{WindowDescriptor, WindowMode},
    },
    ecs::world::World,
    render::{Renderer, error::RenderError},
};

struct KvantumaGame {}

impl Game for KvantumaGame {
    fn init(&mut self, world: &mut World, renderer: &mut Renderer) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
        Ok(())
    }

    fn input(&mut self, event: &glfw::WindowEvent, world: &mut World) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(&mut self, world: &mut World, renderer: &mut Renderer) -> Result<(), RenderError> {
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    App::new(
        WindowDescriptor {
            width: 1280,
            height: 720,
            title: "KVΛNTUMA",
            mode: WindowMode::Windowed,
        }, 
        KvantumaGame {
            // TODO: State
        },
    )?.run();

    Ok(())
}
