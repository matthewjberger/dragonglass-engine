use dragonglass::{
    app::{run_application, App},
    dependencies::{
        anyhow::Result,
        winit::event::{ElementState, Event, VirtualKeyCode},
    },
};

#[derive(Default)]
struct Editor;

impl App for Editor {
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
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

fn main() -> Result<()> {
    run_application(Editor::default(), "Dragonglass Editor")
}
