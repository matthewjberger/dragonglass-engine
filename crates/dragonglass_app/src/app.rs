use std::path::PathBuf;

use dragonglass_dependencies::{
    anyhow::Result,
    egui::CtxRef,
    glutin::{
        dpi::PhysicalSize,
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
        ContextBuilder, ContextWrapper, PossiblyCurrent,
    },
    winit::event::MouseButton,
};
use dragonglass_gui::{Gui, ScreenDescriptor};
use dragonglass_render::{create_render_backend, Backend, Renderer};
use dragonglass_world::{load_gltf, World};

pub trait App {
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    fn update(&mut self) -> Result<()> {
        Ok(())
    }
    fn update_gui(&mut self, _context: CtxRef) -> Result<()> {
        Ok(())
    }
    fn on_file_dropped(&mut self, _path: &PathBuf) -> Result<()> {
        Ok(())
    }
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
    fn on_mouse(&mut self, _button: &MouseButton, _state: &ElementState) -> Result<()> {
        Ok(())
    }
    fn on_key(&mut self, _keycode: &VirtualKeyCode, _keystate: &ElementState) -> Result<()> {
        Ok(())
    }
    fn handle_events(&mut self, _event: Event<()>) -> Result<()> {
        Ok(())
    }
}

pub fn run_application(mut app: impl App + 'static, title: &str) -> Result<()> {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(800, 600));

    let windowed_context = ContextBuilder::new()
        .with_srgb(true)
        .build_windowed(window_builder, &event_loop)?;

    let inner_size = windowed_context.window().inner_size();

    let screen_descriptor = ScreenDescriptor {
        dimensions: inner_size,
        scale_factor: windowed_context.window().scale_factor() as _,
    };
    let mut gui = Gui::new(screen_descriptor);

    let mut context = unsafe { windowed_context.make_current().unwrap() };

    let mut world = World::new()?;
    load_gltf("assets/models/DamagedHelmet.glb", &mut world)?;
    world.add_default_light()?;

    let mut renderer = create_render_backend(&Backend::OpenGL, &context, inner_size)?;
    renderer.load_world(&world)?;

    app.initialize()?;

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = run_loop(
            &mut context,
            &mut app,
            &mut world,
            &mut gui,
            &mut renderer,
            event,
            control_flow,
        ) {
            eprintln!("Application Error: {}", error);
        }
    });
}

fn run_loop(
    context: &ContextWrapper<PossiblyCurrent, Window>,
    app: &mut impl App,
    world: &mut World,
    gui: &mut Gui,
    renderer: &mut Box<dyn Renderer>,
    event: Event<()>,
    control_flow: &mut ControlFlow,
) -> Result<()> {
    *control_flow = ControlFlow::Poll;

    gui.handle_event(&event);

    match event {
        Event::LoopDestroyed => app.cleanup()?,
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::DroppedFile(ref path) => app.on_file_dropped(path)?,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(context, *physical_size);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::MouseInput { button, state, .. } => app.on_mouse(button, state)?,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: keystate,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if let (VirtualKeyCode::Escape, ElementState::Pressed) = (keycode, keystate) {
                    *control_flow = ControlFlow::Exit;
                }
                app.on_key(keycode, keystate)?;
            }
            _ => (),
        },
        Event::MainEventsCleared => {
            app.update()?;

            let _frame_data = gui.start_frame(context.window().scale_factor() as _);
            app.update_gui(gui.context())?;
            let paint_jobs = gui.end_frame(context.window());

            renderer.render(context, world, &paint_jobs)?;
        }
        _ => (),
    }

    app.handle_events(event)?;

    Ok(())
}
