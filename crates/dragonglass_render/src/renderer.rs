use dragonglass_dependencies::{
    anyhow::Result,
    egui::{epaint::ClippedShape, CtxRef},
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};
use dragonglass_world::{Viewport, World};

use crate::vulkan::VulkanRenderDevice;

pub enum Backend {
    Vulkan,
}

pub trait Renderer {
    fn render(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        world: &World,
        gui_context: &CtxRef,
        clipped_shapes: Vec<ClippedShape>,
    ) -> Result<()>;
    fn load_world(&mut self, world: &World) -> Result<()>;
    fn viewport(&self) -> Viewport;
    fn set_viewport(&mut self, viewport: Viewport);
    fn resize(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    );
    fn cleanup(&mut self);
}

pub fn create_render_backend(
    backend: &Backend,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    dimensions: PhysicalSize<u32>,
) -> Result<Box<dyn Renderer>> {
    match backend {
        Backend::Vulkan => {
            let backend = VulkanRenderDevice::new()?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
