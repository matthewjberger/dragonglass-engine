use dragonglass_dependencies::{
    anyhow::Result,
    egui::{epaint::ClippedShape, CtxRef},
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};
use dragonglass_world::{Viewport, World};

use crate::opengl::OpenGLRenderDevice;

pub enum Backend {
    OpenGL,
}

pub trait Renderer {
    fn cleanup(&mut self);
    fn render(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        gui_context: &CtxRef,
        world: &World,
        clipped_shapes: Vec<ClippedShape>,
    ) -> Result<()>;
    fn load_world(&mut self, world: &World) -> Result<()>;
    fn set_viewport(&mut self, viewport: Viewport);
    fn resize(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    );
}

pub fn create_render_backend(
    backend: &Backend,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    dimensions: PhysicalSize<u32>,
) -> Result<Box<dyn Renderer>> {
    match backend {
        Backend::OpenGL => {
            let backend = OpenGLRenderDevice::new(context, dimensions)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
