use crate::{AppState, Input, System};
use dragonglass_dependencies::{
    anyhow::Result,
    glutin::{
        dpi::PhysicalSize,
        event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
};
use dragonglass_gui::{Gui, ScreenDescriptor};
use dragonglass_render::{create_render_backend, Backend};
use dragonglass_world::World;
use std::path::PathBuf;

pub trait App {
    fn initialize(&mut self, _app_state: &mut AppState) -> Result<()> {
        Ok(())
    }
    fn update(&mut self, _app_state: &mut AppState) -> Result<()> {
        Ok(())
    }
    fn gui_active(&mut self) -> bool {
        return false;
    }
    fn update_gui(&mut self, _app_state: &mut AppState) -> Result<()> {
        Ok(())
    }
    fn on_file_dropped(&mut self, _path: &PathBuf, _app_state: &mut AppState) -> Result<()> {
        Ok(())
    }
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
    fn on_mouse(
        &mut self,
        _button: &MouseButton,
        _button_state: &ElementState,
        _app_state: &mut AppState,
    ) -> Result<()> {
        Ok(())
    }
    fn on_key(&mut self, _input: KeyboardInput, _app_state: &mut AppState) -> Result<()> {
        Ok(())
    }
    fn handle_events(&mut self, _event: &Event<()>, _app_state: &mut AppState) -> Result<()> {
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

    let mut renderer = create_render_backend(&Backend::OpenGL, &context, inner_size)?;

    let mut input = Input::default();
    let mut system = System::new(inner_size);

    app.initialize(&mut AppState {
        context: &mut context,
        world: &mut world,
        gui: &mut gui,
        renderer: &mut renderer,
        input: &mut input,
        system: &mut system,
    })?;

    event_loop.run(move |event, _, control_flow| {
        let state = AppState {
            context: &mut context,
            world: &mut world,
            gui: &mut gui,
            renderer: &mut renderer,
            input: &mut input,
            system: &mut system,
        };
        if let Err(error) = run_loop(&mut app, state, event, control_flow) {
            eprintln!("Application Error: {}", error);
        }
    });
}

fn run_loop(
    app: &mut impl App,
    mut app_state: AppState,
    event: Event<()>,
    control_flow: &mut ControlFlow,
) -> Result<()> {
    *control_flow = ControlFlow::Poll;

    if app.gui_active() {
        app_state.gui.handle_event(&event);
    }
    if !app.gui_active() || !app_state.gui.captures_event(&event) {
        app.handle_events(&event, &mut app_state)?;
        app_state.system.handle_event(&event);
        app_state
            .input
            .handle_event(&event, app_state.system.window_center());
    }

    match event {
        Event::NewEvents(_) => {
            if app_state.system.exit_requested {
                *control_flow = ControlFlow::Exit;
            }
        }
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::DroppedFile(ref path) => app.on_file_dropped(path, &mut app_state)?,
            WindowEvent::Resized(physical_size) => {
                app_state.renderer.resize(app_state.context, *physical_size);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::MouseInput { button, state, .. } => {
                app.on_mouse(button, state, &mut app_state)?
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
                    (input.virtual_keycode, input.state)
                {
                    *control_flow = ControlFlow::Exit;
                }
                app.on_key(*input, &mut app_state)?;
            }
            _ => (),
        },
        Event::MainEventsCleared => {
            app_state.world.tick(app_state.system.delta_time as f32)?;
            app.update(&mut app_state)?;

            let clipped_shapes = if app.gui_active() {
                let _frame_data = app_state
                    .gui
                    .start_frame(app_state.context.window().scale_factor() as _);
                app.update_gui(&mut app_state)?;
                app_state.gui.end_frame(app_state.context.window())
            } else {
                Vec::new()
            };

            app_state.renderer.render(
                app_state.context,
                app_state.world,
                &app_state.gui.context(),
                clipped_shapes,
            )?;
        }
        Event::LoopDestroyed => {
            app_state.renderer.cleanup();
            app.cleanup()?;
        }
        _ => (),
    }

    Ok(())
}
