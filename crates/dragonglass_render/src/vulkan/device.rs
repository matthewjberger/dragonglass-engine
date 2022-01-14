use crate::Renderer;
use dragonglass_dependencies::{
    anyhow::Result,
    egui::{epaint::ClippedShape, CtxRef},
    glutin::{self, ContextWrapper, PossiblyCurrent},
    winit::{dpi::PhysicalSize, window::Window},
};
use dragonglass_world::{Viewport, World};

pub(crate) struct VulkanRenderDevice;

impl VulkanRenderDevice {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Renderer for VulkanRenderDevice {
    fn render(
        &mut self,
        context: &glutin::ContextWrapper<glutin::PossiblyCurrent, Window>,
        world: &World,
        gui_context: &CtxRef,
        clipped_shapes: Vec<ClippedShape>,
    ) -> Result<()> {
        todo!()
    }

    fn load_world(&mut self, world: &World) -> Result<()> {
        todo!()
    }

    fn viewport(&self) -> Viewport {
        todo!()
    }

    fn set_viewport(&mut self, viewport: Viewport) {
        todo!()
    }

    fn resize(
        &mut self,
        context: &glutin::ContextWrapper<glutin::PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    ) {
        todo!()
    }

    fn cleanup(&mut self) {
        todo!()
    }
}
