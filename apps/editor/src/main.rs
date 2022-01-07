use dragonglass::{
    app::{run_application, App, AppState, MouseOrbit},
    dependencies::{
        anyhow::Result,
        egui::{self, Id, LayerId, Ui},
        env_logger, log,
    },
    render::Viewport,
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

    fn update_gui(&mut self, app_state: &mut AppState) -> Result<()> {
        let ctx = &app_state.gui.context();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {});
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Left Panel");
                ui.allocate_space(ui.available_size());
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Right Panel");
                ui.allocate_space(ui.available_size());
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Bottom Panel");
                ui.allocate_space(ui.available_size());
            });

        // Calculate the rect needed for rendering
        let viewport = Ui::new(
            ctx.clone(),
            LayerId::background(),
            Id::new("central_panel"),
            ctx.available_rect(),
            ctx.input().screen_rect(),
        )
        .max_rect();

        let dimensions = app_state.context.window().inner_size();
        let offset = dimensions.height as f32 - viewport.max.y;
        app_state.renderer.set_viewport(Viewport {
            x: viewport.min.x,
            y: viewport.min.y,
            width: viewport.width(),
            height: viewport.height(),
            min_depth: -1.0,
            max_depth: 1.0,
        });

        log::info!("viewport: {:?}", viewport);

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
