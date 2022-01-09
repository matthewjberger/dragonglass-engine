use dragonglass_dependencies::{anyhow::Result, nalgebra_glm as glm};
use dragonglass_opengl::{ShaderProgram, Texture};
use dragonglass_world::{Material, World};

use crate::opengl::world::WorldShader;

pub struct SolidShader {
    shader_program: ShaderProgram,
}

impl SolidShader {
    pub fn new() -> Result<Self> {
        let mut shader_program = ShaderProgram::new();
        shader_program
            .vertex_shader_source(VERTEX_SHADER_SOURCE)?
            .fragment_shader_source(FRAGMENT_SHADER_SOURCE)?
            .link();
        Ok(Self { shader_program })
    }

    fn update_uniforms(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        let (projection, view) = world.active_camera_matrices(aspect_ratio)?;
        let camera_entity = world.active_camera()?;
        let camera_transform = world.entity_global_transform(camera_entity)?;
        self.shader_program
            .set_uniform_vec3("cameraPosition", camera_transform.translation.as_slice());
        self.shader_program
            .set_uniform_matrix4x4("projection", projection.as_slice());
        self.shader_program
            .set_uniform_matrix4x4("view", view.as_slice());
        Ok(())
    }
}

impl WorldShader for SolidShader {
    fn update(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        self.shader_program.use_program();
        self.update_uniforms(world, aspect_ratio)?;
        Ok(())
    }

    fn update_model_matrix(&self, model_matrix: glm::Mat4) {
        self.shader_program
            .set_uniform_matrix4x4("model", model_matrix.as_slice());
    }

    fn update_material(&self, _material: &Material, _textures: &[Texture]) -> Result<()> {
        Ok(())
    }
}

const VERTEX_SHADER_SOURCE: &'static str = &r#"
#version 450 core

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inNormal;
layout (location = 2) in vec2 inUV0;
layout (location = 3) in vec2 inUV1;
layout (location = 4) in vec4 inJoint0;
layout (location = 5) in vec4 inWeight0;
layout (location = 6) in vec3 inColor0;

uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;

void main()
{
   vec3 position = vec3(model * vec4(inPosition, 1.0));
   gl_Position = projection * view * vec4(position, 1.0);
}
"#;

const FRAGMENT_SHADER_SOURCE: &'static str = &r#"
#version 450 core

out vec4 color;

void main(void)
{
    color = vec4(0.04, 0.28, 0.26, 1.0);
}
"#;
