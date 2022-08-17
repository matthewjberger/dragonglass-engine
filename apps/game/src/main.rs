use dragonglass::{
    app::{run_application, App, AppConfig, AppState, MouseLook},
    dependencies::{
        anyhow::{Context, Result},
        legion::IntoQuery,
        log, nalgebra_glm as glm,
        rapier3d::prelude::{InteractionGroups, RigidBodyBuilder, RigidBodyType},
        winit::event::{ElementState, VirtualKeyCode},
    },
    world::{
        Camera as WorldCamera, Entity, EntityStore, MeshRender, PerspectiveCamera, Projection,
        RigidBody, Transform, Vector3,
    },
};

const PLAYER_COLLISION_GROUP: InteractionGroups = InteractionGroups::new(0b10, 0b01);
const LEVEL_COLLISION_GROUP: InteractionGroups = InteractionGroups::new(0b01, 0b10);

#[derive(Default)]
struct Game {
    player: Option<Entity>,
    camera: MouseLook,
}

impl App for Game {
    fn initialize(&mut self, app_state: &mut dragonglass::app::AppState) -> Result<()> {
        app_state.world.physics.set_gravity(Vector3::y() * -14.0);

        app_state.set_fullscreen();
        self.camera.orientation.sensitivity = glm::vec2(0.5, 0.5);

        // Load player
        let position = glm::vec3(0.0, 1.0, 0.0);
        let transform = Transform {
            translation: position,
            ..Default::default()
        };

        {
            let player_entity = app_state.world.ecs.push((transform,));
            app_state
                .world
                .scene
                .default_scenegraph_mut()?
                .add_node(player_entity);
            self.player = Some(player_entity);
        }

        app_state.load_asset("assets/models/gamemap.glb")?;

        let mut level_meshes = Vec::new();
        let mut query = <(Entity, &MeshRender)>::query();
        for (entity, mesh) in query.iter(&app_state.world.ecs) {
            level_meshes.push(*entity);
            log::info!("Mesh available: {}", mesh.name);
        }
        for entity in level_meshes.into_iter() {
            app_state
                .world
                .add_rigid_body(entity, RigidBodyType::Fixed)?;
            // add_box_collider(application, entity, LEVEL_COLLISION_GROUP)?;
            app_state
                .world
                .add_trimesh_collider(entity, LEVEL_COLLISION_GROUP)?;
        }

        // Setup player
        if let Some(entity) = self.player.as_ref() {
            activate_first_person(app_state, *entity)?;
            let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .translation(transform.translation)
                .lock_rotations()
                .build();
            let handle = app_state.world.physics.bodies.insert(rigid_body);
            app_state
                .world
                .ecs
                .entry(*entity)
                .context("")?
                .add_component(RigidBody::new(handle));

            app_state
                .world
                .add_cylinder_collider(*entity, 1.0, 0.5, PLAYER_COLLISION_GROUP)?;
        }

        Ok(())
    }

    fn update(&mut self, app_state: &mut AppState) -> Result<()> {
        if let Some(player) = self.player.as_ref() {
            self.camera.update(app_state, *player)?;
            update_player(app_state, *player)?;
        }
        Ok(())
    }

    fn on_key(
        &mut self,
        input: dragonglass::dependencies::winit::event::KeyboardInput,
        app_state: &mut AppState,
    ) -> Result<()> {
        if let (Some(VirtualKeyCode::Space), ElementState::Pressed) =
            (input.virtual_keycode, input.state)
        {
            if let Some(player) = self.player.as_ref() {
                jump_player(app_state, *player)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    run_application(
        Game::default(),
        &AppConfig {
            icon: Some("assets/icon/icon.png".to_string()),
            title: "Example Game".to_string(),
            ..Default::default()
        },
    )
}

fn update_player(app_state: &mut AppState, entity: Entity) -> Result<()> {
    let speed = 4.0 * app_state.system.delta_time as f32;
    {
        let mut entry = app_state.world.ecs.entry_mut(entity)?;
        let transform = entry.get_component_mut::<Transform>()?;
        let mut translation = glm::vec3(0.0, 0.0, 0.0);

        if app_state.input.is_key_pressed(VirtualKeyCode::W) {
            translation = speed * transform.forward();
        }

        if app_state.input.is_key_pressed(VirtualKeyCode::A) {
            translation = -speed * transform.right();
        }

        if app_state.input.is_key_pressed(VirtualKeyCode::S) {
            translation = -speed * transform.forward();
        }

        if app_state.input.is_key_pressed(VirtualKeyCode::D) {
            translation = speed * transform.right();
        }

        transform.translation += translation;
    }
    app_state.world.sync_rigid_body_to_transform(entity)?;
    Ok(())
}

fn jump_player(app_state: &mut AppState, entity: Entity) -> Result<()> {
    let rigid_body_handle = app_state
        .world
        .ecs
        .entry_ref(entity)?
        .get_component::<RigidBody>()?
        .handle;
    if let Some(rigid_body) = app_state.world.physics.bodies.get_mut(rigid_body_handle) {
        let jump_strength = 10.0;
        let impulse = jump_strength * glm::Vec3::y();
        rigid_body.apply_impulse(impulse, true);
    }
    app_state.world.sync_transform_to_rigid_body(entity)?;
    Ok(())
}

fn activate_first_person(app_state: &mut AppState, entity: Entity) -> Result<()> {
    // Disable active camera
    let camera_entity = app_state.world.active_camera()?;
    app_state
        .world
        .ecs
        .entry_mut(camera_entity)?
        .get_component_mut::<WorldCamera>()?
        .enabled = false;

    app_state
        .world
        .ecs
        .entry(entity)
        .context("entity not found")?
        .add_component(WorldCamera {
            name: "Player Camera".to_string(),
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 90_f32.to_radians(),
                z_far: Some(1000.0),
                z_near: 0.001,
            }),
            enabled: true,
        });

    Ok(())
}
