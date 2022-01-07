use dragonglass_dependencies::{
    anyhow::Result,
    glutin::{window::Window, ContextWrapper, NotCurrent},
    winit::dpi::PhysicalSize,
};
use dragonglass_world::World;

use crate::opengl::OpenGLRenderDevice;

pub enum Backend {
    OpenGL,
}

#[derive(Default)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        let height = if self.height > 0.0 { self.height } else { 1.0 };
        self.width / height
    }
}

pub trait Renderer {
    fn render(&mut self) -> Result<()>;
    fn load_world(&mut self, world: &World) -> Result<()>;
    fn set_viewport(&mut self, viewport: Viewport);
    fn resize(&mut self, dimensions: PhysicalSize<u32>);
}

pub fn create_render_backend(
    backend: &Backend,
    context: ContextWrapper<NotCurrent, Window>,
    dimensions: PhysicalSize<u32>,
) -> Result<Box<dyn Renderer>> {
    match backend {
        Backend::OpenGL => {
            let backend = OpenGLRenderDevice::new(context, dimensions)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
