use dragonglass_dependencies::{
    anyhow::Result,
    gl,
    glutin::{window::Window, ContextWrapper, NotCurrent, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};

use crate::Renderer;

pub struct OpenGLRenderDevice {
    context: ContextWrapper<PossiblyCurrent, Window>,
}

impl OpenGLRenderDevice {
    pub fn new(
        context: ContextWrapper<NotCurrent, Window>,
        _dimensions: PhysicalSize<u32>,
    ) -> Result<Self> {
        let context = unsafe { context.make_current().unwrap() };
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        Ok(Self { context })
    }
}

impl Renderer for OpenGLRenderDevice {
    fn render(&mut self) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        }
        self.context.swap_buffers()?;
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

    fn resize(&mut self, dimensions: PhysicalSize<u32>) {
        self.context.resize(dimensions);
        unsafe {
            gl::Viewport(0, 0, dimensions.width as _, dimensions.height as _);
        }
    }
}
