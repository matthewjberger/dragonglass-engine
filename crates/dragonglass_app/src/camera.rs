use crate::AppState;
use dragonglass_dependencies::{anyhow::Result, nalgebra_glm as glm};
use dragonglass_world::{Entity, EntityStore, Transform};

#[derive(Default)]
pub struct MouseOrbit {
    pub orientation: Orientation,
}

impl MouseOrbit {
    pub fn update(&mut self, app_state: &mut AppState, entity: Entity) -> Result<()> {
        self.orientation
            .zoom(app_state.input.mouse.wheel_delta.y * 0.3);

        let mouse_delta = app_state.input.mouse.position_delta * app_state.system.delta_time as f32;

        if app_state.input.mouse.is_left_clicked {
            let mut delta = mouse_delta;
            delta.x = -1.0 * mouse_delta.x;
            self.orientation.rotate(&delta);
        }

        {
            let mut entry = app_state.world.ecs.entry_mut(entity)?;
            let mut transform = entry.get_component_mut::<Transform>()?;
            if app_state.input.mouse.is_right_clicked {
                self.orientation.pan(&mouse_delta)
            }
            transform.translation = self.orientation.position();
            transform.rotation = self.orientation.look_at_offset();
        }

        app_state.set_cursor_grab(false)?;
        app_state.set_cursor_visible(true);

        Ok(())
    }
}

#[derive(Default)]
pub struct MouseLook {
    pub orientation: Orientation,
}

impl MouseLook {
    pub fn update(&mut self, app_state: &mut AppState, entity: Entity) -> Result<()> {
        let mouse_delta =
            app_state.input.mouse.offset_from_center * app_state.system.delta_time as f32;

        self.orientation.rotate(&mouse_delta);

        {
            let mut entry = app_state.world.ecs.entry_mut(entity)?;
            let mut transform = entry.get_component_mut::<Transform>()?;
            transform.rotation = self.orientation.look_forward();
        }

        app_state.set_cursor_grab(true)?;
        app_state.set_cursor_visible(false);
        app_state.center_cursor()?;

        Ok(())
    }
}

pub struct Orientation {
    pub min_radius: f32,
    pub max_radius: f32,
    pub radius: f32,
    pub offset: glm::Vec3,
    pub sensitivity: glm::Vec2,
    pub direction: glm::Vec2,
}

impl Orientation {
    pub fn direction(&self) -> glm::Vec3 {
        glm::vec3(
            self.direction.y.sin() * self.direction.x.sin(),
            self.direction.y.cos(),
            self.direction.y.sin() * self.direction.x.cos(),
        )
    }

    pub fn rotate(&mut self, position_delta: &glm::Vec2) {
        let delta = position_delta.component_mul(&self.sensitivity);
        self.direction.x += delta.x;
        self.direction.y = glm::clamp_scalar(
            self.direction.y + delta.y,
            10.0_f32.to_radians(),
            170.0_f32.to_radians(),
        );
    }

    pub fn up(&self) -> glm::Vec3 {
        self.right().cross(&self.direction())
    }

    pub fn right(&self) -> glm::Vec3 {
        self.direction().cross(&glm::Vec3::y()).normalize()
    }

    pub fn pan(&mut self, offset: &glm::Vec2) {
        self.offset += self.right() * offset.x;
        self.offset += self.up() * offset.y;
    }

    pub fn position(&self) -> glm::Vec3 {
        (self.direction() * self.radius) + self.offset
    }

    pub fn zoom(&mut self, distance: f32) {
        self.radius -= distance;
        if self.radius < self.min_radius {
            self.radius = self.min_radius;
        }
        if self.radius > self.max_radius {
            self.radius = self.max_radius;
        }
    }

    pub fn look_at_offset(&self) -> glm::Quat {
        self.look(self.offset - self.position())
    }

    pub fn look_forward(&self) -> glm::Quat {
        self.look(-self.direction())
    }

    fn look(&self, point: glm::Vec3) -> glm::Quat {
        glm::quat_conjugate(&glm::quat_look_at(&point, &glm::Vec3::y()))
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            min_radius: 1.0,
            max_radius: 100.0,
            radius: 5.0,
            offset: glm::vec3(0.0, 0.0, 0.0),
            sensitivity: glm::vec2(1.0, 1.0),
            direction: glm::vec2(0_f32.to_radians(), 45_f32.to_radians()),
        }
    }
}
