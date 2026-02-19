use glam::{IVec2, UVec2};
use glfw::Window;

use crate::{error::GameError, render::error::RenderError};

pub mod error;

pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: UVec2,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Renderer, GameError> {
        let size = IVec2::from(window.get_framebuffer_size()).as_uvec2();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, 
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(&window)
                .map_err(|e| RenderError::HandleError(e.to_string()))?;
            instance.create_surface_unsafe(target)
                .map_err(RenderError::from)?
        };

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        
        let adapter = instance.request_adapter(&adapter_descriptor).await
            .map_err(RenderError::from)?;

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Logical device"),
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(&device_descriptor).await
            .map_err(RenderError::from)?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats
            .iter()
            .copied()
            .find(|f | f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.x,
            height: size.y,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);

        Ok(Renderer { 
            instance, 
            surface, 
            device, 
            queue, 
            config, 
            size,
        })
    }
}