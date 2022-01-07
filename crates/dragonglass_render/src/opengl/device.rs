use crate::{opengl::world::WorldRender, Renderer, Viewport};
use dragonglass_dependencies::{
    anyhow::Result,
    egui::{epaint::ClippedShape, CtxRef},
    egui_glow, gl, glow,
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    winit::dpi::PhysicalSize,
};
use dragonglass_world::World;

pub struct OpenGLRenderDevice {
    world_render: Option<WorldRender>,
    glow: glow::Context,
    egui_glow: egui_glow::EguiGlow,
    viewport: Viewport,
}

impl OpenGLRenderDevice {
    pub fn new(
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    ) -> Result<Self> {
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        let glow_context = unsafe {
            glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol))
        };
        let egui_glow = egui_glow::EguiGlow::new(context, &glow_context);
        Ok(Self {
            world_render: None,
            glow: glow_context,
            egui_glow,
            viewport: Viewport {
                x: 0.0,
                y: 0.0,
                width: dimensions.width as _,
                height: dimensions.height as _,
                min_depth: -1.0,
                max_depth: 1.0,
            },
        })
    }
}

impl Renderer for OpenGLRenderDevice {
    fn cleanup(&mut self) {
        self.egui_glow.painter.destroy(&self.glow);
    }

    fn render(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        gui_context: &CtxRef,
        world: &World,
        clipped_shapes: Vec<ClippedShape>,
    ) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        }

        let aspect_ratio =
            self.viewport.width as f32 / std::cmp::max(self.viewport.height as u32, 1) as f32;

        if let Some(world_render) = self.world_render.as_ref() {
            world_render.render(world, aspect_ratio)?;
        }

        unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB) };
        let clipped_meshes = gui_context.tessellate(clipped_shapes);
        let dimensions: [u32; 2] = context.window().inner_size().into();
        self.egui_glow
            .painter
            .upload_egui_texture(&self.glow, &gui_context.font_image());
        self.egui_glow.painter.paint_meshes(
            &self.glow,
            dimensions,
            gui_context.pixels_per_point(),
            clipped_meshes,
        );

        context.swap_buffers()?;

        Ok(())
    }

    fn load_world(&mut self, world: &World) -> Result<()> {
        self.world_render = Some(WorldRender::new(world)?);
        Ok(())
    }

    fn set_viewport(&mut self, viewport: Viewport) {
        unsafe {
            gl::Viewport(
                viewport.x as _,
                viewport.y as _,
                viewport.width as _,
                viewport.height as _,
            );
        }
        self.viewport = viewport;
    }

    fn resize(
        &mut self,
        context: &ContextWrapper<PossiblyCurrent, Window>,
        dimensions: PhysicalSize<u32>,
    ) {
        context.resize(dimensions);
    }
}
