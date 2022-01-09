use dragonglass::{
    app::{run_application, App, AppState, MouseOrbit},
    dependencies::{
        anyhow::Result,
        egui::{self, global_dark_light_mode_switch, Id, LayerId, Ui},
        env_logger,
        legion::IntoQuery,
        log,
        rapier3d::prelude::{InteractionGroups, RigidBodyType},
        winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
    },
    world::{load_gltf, Entity, MeshRender, Viewport},
};

#[derive(Default)]
struct Editor {
    camera: MouseOrbit,
}

impl App for Editor {
    fn initialize(&mut self, world: &mut dragonglass::world::World) -> Result<()> {
        env_logger::init();
        world.add_default_light()?;
        Ok(())
    }

    fn update(&mut self, app_state: &mut AppState) -> Result<()> {
        if app_state.world.active_camera_is_main()? {
            let camera_entity = app_state.world.active_camera()?;
            self.camera.update(app_state, camera_entity)?;
        }

        // Animate entities
        // if !app_state.world.animations.is_empty() {
        //     app_state
        //         .world
        //         .animate(0, 0.75 * app_state.system.delta_time as f32)?;
        // }

        Ok(())
    }

    fn update_gui(&mut self, app_state: &mut AppState) -> Result<()> {
        let ctx = &app_state.gui.context();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    global_dark_light_mode_switch(ui);
                });
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

        // This is the space leftover on screen after the UI is drawn
        // We can restrict rendering to this viewport to
        // prevent drawing the gui over the scene
        let central_rect = Ui::new(
            ctx.clone(),
            LayerId::background(),
            Id::new("central_panel"),
            ctx.available_rect(),
            ctx.input().screen_rect(),
        )
        .max_rect();

        // TODO: Don't render underneath the gui
        let _viewport = Viewport {
            x: central_rect.min.x,
            y: central_rect.min.y,
            width: central_rect.width(),
            height: central_rect.height(),
        };
        // app_state.renderer.set_viewport(viewport);

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

                    let mut query = <(Entity, &MeshRender)>::query();
                    let entities = query
                        .iter(&mut app_state.world.ecs)
                        .map(|(e, _)| *e)
                        .collect::<Vec<_>>();

                    for entity in entities.into_iter() {
                        app_state
                            .world
                            .add_rigid_body(entity, RigidBodyType::Static)?;
                        app_state
                            .world
                            .add_trimesh_collider(entity, InteractionGroups::all())?;
                    }
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

        Ok(())
    }

    fn on_mouse(
        &mut self,
        button: &MouseButton,
        button_state: &ElementState,
        app_state: &mut AppState,
    ) -> Result<()> {
        if (MouseButton::Left, ElementState::Pressed) == (*button, *button_state) {
            let interact_distance = f32::MAX;
            if let Some(entity) = app_state.world.pick_object(
                &app_state.mouse_ray_configuration()?,
                interact_distance,
                InteractionGroups::all(),
            )? {
                log::info!("Picked entity: {:?}", entity);
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    run_application(Editor::default(), "Dragonglass Editor")
}
