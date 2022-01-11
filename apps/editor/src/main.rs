use std::path::PathBuf;

use dragonglass::{
    app::{run_application, App, AppState, MouseOrbit},
    dependencies::{
        anyhow::{Context, Result},
        egui::{
            self, global_dark_light_mode_switch, menu, Align, DragValue, Id, LayerId,
            SelectableLabel, Ui,
        },
        env_logger,
        legion::IntoQuery,
        log,
        petgraph::{graph::NodeIndex, EdgeDirection::Outgoing},
        rapier3d::prelude::{InteractionGroups, RigidBodyType},
        rfd::FileDialog,
        winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
    },
    world::{
        load_gltf, Ecs, Entity, EntityStore, MeshRender, Name, SceneGraph, Selected, Transform,
        Viewport,
    },
};

#[derive(Default)]
struct Editor {
    camera: MouseOrbit,
    moving_selected: bool,
    selected_entity: Option<Entity>,
}

impl Editor {
    pub fn deselect_all(&mut self, app_state: &mut AppState) -> Result<()> {
        let mut query = <(Entity, &Selected)>::query();

        let entities = query
            .iter(&mut app_state.world.ecs)
            .map(|(e, _)| *e)
            .collect::<Vec<_>>();

        for entity in entities.into_iter() {
            let mut entry = app_state
                .world
                .ecs
                .entry(entity)
                .context("Failed to find entity!")?;
            log::info!("Deselecting entity: {:?}", entity);
            entry.remove_component::<Selected>();
        }

        Ok(())
    }

    pub fn load_world_from_file(&self, path: &PathBuf, app_state: &mut AppState) -> Result<()> {
        let raw_path = match path.to_str() {
            Some(raw_path) => raw_path,
            None => return Ok(()),
        };

        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("glb") | Some("gltf") => {
                    load_gltf(raw_path, app_state.world)?;
                }
                // Some("hdr") => Self::load_hdr(raw_path, application)?,
                Some("dga") => {
                    app_state.world.load(raw_path)?;
                    log::info!("Loaded world!");
                }
                _ => log::warn!(
                    "File extension {:#?} is not a valid '.dga', '.glb', '.gltf', or '.hdr' extension",
                    extension
                ),
            }

            // TODO: Probably don't want this added every time
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

        Ok(())
    }

    fn print_node(&mut self, ecs: &mut Ecs, graph: &SceneGraph, index: NodeIndex, ui: &mut Ui) {
        let entity = graph[index];
        let entry = ecs.entry_ref(entity).expect("Failed to find entity!");
        let debug_name = format!("{:?}", entity);
        let label = entry
            .get_component::<Name>()
            .ok()
            .unwrap_or(&Name(debug_name))
            .0
            .to_string();

        let selected = self.selected_entity == Some(entity);

        let response = if graph.has_children(index) {
            egui::CollapsingHeader::new(label.to_string())
                .selectable(true)
                .selected(selected)
                .show(ui, |ui| {
                    let mut neighbors = graph.neighbors(index, Outgoing);
                    while let Some(child) = neighbors.next_node(&graph.0) {
                        self.print_node(ecs, graph, child, ui);
                    }
                })
                .header_response
        } else {
            ui.add(SelectableLabel::new(selected, label.to_string()))
        };

        if response.clicked() {
            self.selected_entity = Some(entity);
        }
    }
}

impl App for Editor {
    fn gui_active(&mut self) -> bool {
        true
    }

    fn initialize(&mut self, _app_state: &mut AppState) -> Result<()> {
        env_logger::init();
        Ok(())
    }

    fn update(&mut self, app_state: &mut AppState) -> Result<()> {
        if app_state.world.active_camera_is_main()? && !self.moving_selected {
            let camera_entity = app_state.world.active_camera()?;
            self.camera.update(app_state, camera_entity)?;
        }

        // Animate entities
        // if !app_state.world.animations.is_empty() {
        //     app_state
        //         .world
        //         .animate(0, 0.75 * app_state.system.delta_time as f32)?;
        // }

        if self.moving_selected {
            let mut query = <(Entity, &Selected)>::query();
            let entities = query
                .iter_mut(&mut app_state.world.ecs)
                .map(|(e, _)| (*e))
                .collect::<Vec<_>>();
            for entity in entities.into_iter() {
                let mut entry = app_state.world.ecs.entry_mut(entity)?;
                let speed = 10.0;
                let transform = entry.get_component_mut::<Transform>()?;
                let mouse_delta =
                    app_state.input.mouse.position_delta * app_state.system.delta_time as f32;
                if app_state.input.mouse.is_right_clicked {
                    transform.translation += transform.right() * mouse_delta.x * speed;
                    transform.translation += transform.up() * -mouse_delta.y * speed;
                }
                app_state.world.sync_rigid_body_to_transform(entity)?;
            }
        }

        Ok(())
    }

    fn update_gui(&mut self, app_state: &mut AppState) -> Result<()> {
        let ctx = &app_state.gui.context();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    menu::bar(ui, |ui| {
                        global_dark_light_mode_switch(ui);
                        ui.menu_button("File", |ui| {
                            if ui.button("Open / Import").clicked() {
                                let path = FileDialog::new()
                                    .add_filter("dragonglass_asset", &["dga"])
                                    .set_directory("/")
                                    .pick_file();
                                if let Some(path) = path {
                                    self.load_world_from_file(&path, app_state)
                                        .expect("Failed to load asset!");
                                }
                                ui.close_menu();
                            }

                            if ui.button("Quit").clicked() {
                                app_state.system.exit_requested = true;
                            }
                        });
                    });
                });
            });

        egui::SidePanel::left("scene_explorer")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading(&app_state.world.scene.name);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let scene = &mut app_state.world.scene;
                    let ecs = &mut app_state.world.ecs;
                    for graph in scene.graphs.iter_mut() {
                        self.print_node(ecs, graph, NodeIndex::new(0), ui);
                    }
                    ui.allocate_space(ui.available_size());
                });
            });

        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                if let Some(entity) = self.selected_entity {
                    ui.heading("Transform");

                    let mut should_sync = false;

                    ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                        let mut entry = app_state
                            .world
                            .ecs
                            .entry(entity)
                            .expect("Failed to find entity!");

                        let transform = entry
                            .get_component_mut::<Transform>()
                            .expect("Entity does not have a transform!");

                        ui.label("X");
                        let x_response =
                            ui.add(DragValue::new(&mut transform.translation.x).speed(0.1));

                        ui.label("Y");
                        let y_response =
                            ui.add(DragValue::new(&mut transform.translation.y).speed(0.1));

                        ui.label("Z");
                        let z_response =
                            ui.add(DragValue::new(&mut transform.translation.z).speed(0.1));

                        should_sync =
                            x_response.changed() || y_response.changed() || z_response.changed();
                    });

                    if should_sync {
                        app_state
                            .world
                            .sync_rigid_body_to_transform(entity)
                            .expect("Failed to sync rigid body to transform!");
                    }
                }

                ui.allocate_space(ui.available_size());
            });

        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Console");
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
        self.load_world_from_file(path, app_state)?;
        Ok(())
    }

    fn on_key(&mut self, input: KeyboardInput, app_state: &mut AppState) -> Result<()> {
        if input.virtual_keycode == Some(VirtualKeyCode::C) && input.state == ElementState::Pressed
        {
            app_state.world.clear()?;
        }

        if input.virtual_keycode == Some(VirtualKeyCode::G) && input.state == ElementState::Pressed
        {
            self.moving_selected = !self.moving_selected;
        }

        // TODO: Move this into the gui
        if input.virtual_keycode == Some(VirtualKeyCode::S) && input.state == ElementState::Pressed
        {
            app_state.world.save("map.dga")?;
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
                let mut query = <(Entity, &Selected)>::query();
                let already_selected = query
                    .iter(&mut app_state.world.ecs)
                    .map(|(e, _)| *e)
                    .any(|e| e == entity);
                if already_selected {
                    return Ok(());
                }

                self.deselect_all(app_state)?;
                let mut entry = app_state
                    .world
                    .ecs
                    .entry(entity)
                    .context("Failed to find entity")?;
                entry.add_component(Selected::default());
                self.selected_entity = Some(entity);
                log::info!("Selected entity: {:?}", entity);
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    run_application(Editor::default(), "Dragonglass Editor")
}
