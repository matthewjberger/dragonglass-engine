use dragonglass_dependencies::{
    anyhow::Result,
    egui::ClippedMesh,
    gl,
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};

use crate::Renderer;

pub struct OpenGLRenderDevice;

impl OpenGLRenderDevice {
    pub fn new(
        context: &ContextWrapper<PossiblyCurrent, Window>,
        _dimensions: PhysicalSize<u32>,
    ) -> Result<Self> {
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        Ok(Self {})
    }
}

impl Renderer for OpenGLRenderDevice {
    fn render(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        _paint_jobs: &[ClippedMesh],
    ) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        }
        context.swap_buffers()?;
        Ok(())
    }

    fn load_world(
        &mut self,
        _world: &dragonglass_world::World,
    ) -> dragonglass_dependencies::anyhow::Result<()> {
        todo!()
    }

    fn set_viewport(&mut self, _viewport: crate::renderer::Viewport) {
        todo!()
    }

    fn resize(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    ) {
        context.resize(dimensions);
        unsafe {
            gl::Viewport(0, 0, dimensions.width as _, dimensions.height as _);
        }
    }
}
