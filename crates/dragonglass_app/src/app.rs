use dragonglass_dependencies::{
    anyhow::Result,
    egui::CtxRef,
    glutin::{
        dpi::PhysicalSize,
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
};
use dragonglass_gui::{Gui, ScreenDescriptor};
use dragonglass_render::{create_render_backend, Backend, Renderer};

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
    fn cleanup(&mut self) -> Result<()> {
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

    let mut renderer = create_render_backend(&Backend::OpenGL, windowed_context, inner_size)?;

    app.initialize()?;

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = run_loop(&mut app, &mut gui, &mut renderer, event, control_flow) {
            eprintln!("Application Error: {}", error);
        }
    });
}

fn run_loop(
    app: &mut impl App,
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
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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

            let _frame_data = gui.start_frame(1.0);
            app.update_gui(gui.context())?;
            let paint_jobs = gui.end_frame();

            renderer.render(&paint_jobs)?;
        }
        _ => (),
    }

    app.handle_events(event)?;

    Ok(())
}
