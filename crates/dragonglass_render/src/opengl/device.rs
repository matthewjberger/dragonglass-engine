use crate::{opengl::world::WorldRender, Renderer};
use dragonglass_dependencies::{
    anyhow::Result,
    egui::ClippedMesh,
    gl,
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};
use dragonglass_world::World;

pub struct OpenGLRenderDevice {
    world_render: Option<WorldRender>,
}

impl OpenGLRenderDevice {
    pub fn new(
        context: &ContextWrapper<PossiblyCurrent, Window>,
        _dimensions: PhysicalSize<u32>,
    ) -> Result<Self> {
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        Ok(Self { world_render: None })
    }
}

impl Renderer for OpenGLRenderDevice {
    fn render(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        world: &World,
        _paint_jobs: &[ClippedMesh],
    ) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        }

        let dimensions = context.window().inner_size();
        let aspect_ratio =
            dimensions.width as f32 / std::cmp::max(dimensions.height as u32, 1) as f32;

        if let Some(world_render) = self.world_render.as_ref() {
            world_render.render(world, aspect_ratio)?;
        }

        context.swap_buffers()?;

        Ok(())
    }

    fn load_world(&mut self, world: &World) -> Result<()> {
        self.world_render = Some(WorldRender::new(world)?);
        Ok(())
    }

    fn set_viewport(&mut self, _viewport: crate::renderer::Viewport) {
        todo!()
    }

    fn resize(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    ) {
        unsafe {
            gl::Viewport(0, 0, dimensions.width as _, dimensions.height as _);
        }
        context.resize(dimensions);
    }

    fn cleanup(&mut self) {
        self.world_render = None;
    }
}
