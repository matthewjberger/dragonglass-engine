use dragonglass::{
    app::{run_application, App},
    dependencies::{
        anyhow::Result,
        log,
        winit::event::{ElementState, Event, VirtualKeyCode},
    },
    world::{load_gltf, World},
};
use std::path;

#[derive(Default)]
struct Editor;

impl App for Editor {
    fn on_file_dropped(&mut self, path: &std::path::PathBuf) -> Result<()> {
        let _raw_path = match path.to_str() {
            Some(raw_path) => raw_path,
            None => return Ok(()),
        };

        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("glb") | Some("gltf") => { /* TODO: Load gltf */ },
                // Some("hdr") => Self::load_hdr(raw_path, application)?,
                Some("dga") => {
                    // application.world = World::load(raw_path)?;
                    // application.renderer.load_world(&application.world)?;
                    log::info!("Loaded world!");
                }
                _ => log::warn!(
                    "File extension {:#?} is not a valid '.dga', '.glb', '.gltf', or '.hdr' extension",
                    extension
                ),
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    run_application(Editor::default(), "Dragonglass Editor")
}
