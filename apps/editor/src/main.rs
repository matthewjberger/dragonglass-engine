use dragonglass::{
    app::{run_application, App, AppState, MouseOrbit},
    dependencies::{anyhow::Result, env_logger, log},
    world::load_gltf,
};

#[derive(Default)]
struct Editor {
    camera: MouseOrbit,
}

impl App for Editor {
    fn initialize(&mut self, world: &mut dragonglass::world::World) -> Result<()> {
        env_logger::init();
        world.add_default_light()
    }

    fn update(&mut self, app_state: &mut AppState) -> Result<()> {
        if app_state.world.active_camera_is_main()? {
            let camera_entity = app_state.world.active_camera()?;
            self.camera.update(app_state, camera_entity)?;
        }
        Ok(())
    }

    fn on_file_dropped(
        &mut self,
        path: &std::path::PathBuf,
        app_state: &mut AppState,
    ) -> Result<()> {
        let raw_path = match path.to_str() {
            Some(raw_path) => raw_path,
            None => return Ok(()),
        };

        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("glb") | Some("gltf") => {
                    load_gltf(raw_path, app_state.world)?;
                    app_state.renderer.load_world(app_state.world)?;
                }
                // Some("hdr") => Self::load_hdr(raw_path, application)?,
                Some("dga") => {
                    // TODO: Load from dga
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
