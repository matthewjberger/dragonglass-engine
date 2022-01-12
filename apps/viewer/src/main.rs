use dragonglass::{
    app::{run_application, App, AppConfig, AppState, MouseOrbit},
    dependencies::{
        anyhow::Result,
        egui::{self, Id, LayerId, Ui},
        env_logger,
        legion::IntoQuery,
        log,
        winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
    },
    world::{load_gltf, Camera, Entity, Viewport},
};

#[derive(Default)]
struct Viewer {
    camera: MouseOrbit,
}

impl App for Viewer {
    fn gui_active(&mut self) -> bool {
        true
    }

    fn initialize(&mut self, app_state: &mut AppState) -> Result<()> {
        env_logger::init();
        app_state.world.add_default_light()?;
        Ok(())
    }

    fn update(&mut self, app_state: &mut AppState) -> Result<()> {
        if app_state.world.active_camera_is_main()? {
            let camera_entity = app_state.world.active_camera()?;
            self.camera.update(app_state, camera_entity)?;
        }

        if !app_state.world.animations.is_empty() {
            app_state
                .world
                .animate(0, 0.75 * app_state.system.delta_time as f32)?;
        }

        Ok(())
    }

    fn update_gui(&mut self, app_state: &mut AppState) -> Result<()> {
        let ctx = &app_state.gui.context();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |_ui| {});
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Left Panel");
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
        let _offset = dimensions.height as f32 - viewport.max.y;
        app_state.renderer.set_viewport(Viewport {
            x: viewport.min.x,
            y: viewport.min.y,
            width: viewport.width(),
            height: viewport.height(),
        });

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
                    app_state.world.add_default_light()?;
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

    fn on_key(&mut self, input: KeyboardInput, app_state: &mut AppState) -> Result<()> {
        if input.virtual_keycode == Some(VirtualKeyCode::C) && input.state == ElementState::Pressed
        {
            app_state.world.clear()?;
        }

        if input.virtual_keycode == Some(VirtualKeyCode::Space) {
            let mut query = <(Entity, &mut Camera)>::query();
            for (index, (_entity, camera)) in query.iter_mut(&mut app_state.world.ecs).enumerate() {
                camera.enabled = index == 7;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    run_application(
        Viewer::default(),
        &AppConfig {
            icon: Some("assets/icon/icon.png".to_string()),
            title: "Dragonglass Viewer".to_string(),
            ..Default::default()
        },
    )
}
