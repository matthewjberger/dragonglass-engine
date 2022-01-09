use super::pbr::PbrShaderProgram;
use dragonglass_dependencies::{anyhow::Result, gl, nalgebra_glm as glm};
use dragonglass_opengl::{GeometryBuffer, Texture};
use dragonglass_world::{
    AlphaMode, EntityStore, LightKind, Material, Mesh, MeshRender, TextureFormat, Transform, World,
};
use std::ptr;

// TODO: This is duplicated in the vulkan backend and should be moved
#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Light {
    pub direction: glm::Vec3,
    pub range: f32,

    pub color: glm::Vec3,
    pub intensity: f32,

    pub position: glm::Vec3,
    pub inner_cone_cos: f32,

    pub outer_cone_cos: f32,
    pub kind: i32,

    pub _padding: glm::Vec2,
}

impl Light {
    pub fn from_node(transform: &Transform, light: &dragonglass_world::Light) -> Self {
        let mut inner_cone_cos: f32 = 0.0;
        let mut outer_cone_cos: f32 = 0.0;
        let kind = match light.kind {
            LightKind::Directional => 0,
            LightKind::Point => 1,
            LightKind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                inner_cone_cos = inner_cone_angle;
                outer_cone_cos = outer_cone_angle;
                2
            }
        };
        Self {
            direction: -1.0 * glm::quat_rotate_vec3(&transform.rotation, &glm::Vec3::z()),
            range: light.range,
            color: light.color,
            intensity: light.intensity,
            position: transform.translation,
            inner_cone_cos,
            outer_cone_cos,
            kind,
            _padding: glm::vec2(0.0, 0.0),
        }
    }
}

pub trait WorldShader {
    fn update(&self, world: &World, aspect_ratio: f32) -> Result<()>;
    fn update_model_matrix(&self, model_matrix: glm::Mat4);
    fn update_material(&self, material: &Material, textures: &[Texture]) -> Result<()>;
}

pub struct WorldRender {
    pub geometry: GeometryBuffer,
    pub textures: Vec<Texture>,
    pub pbr_shader: PbrShaderProgram,
}

impl WorldRender {
    pub fn new(world: &World) -> Result<Self> {
        let geometry = GeometryBuffer::new(
            &world.geometry.vertices,
            Some(&world.geometry.indices),
            &[3, 3, 2, 2, 4, 4, 3],
        );

        let textures = world
            .textures
            .iter()
            .map(Self::map_world_texture)
            .collect::<Vec<_>>();

        let pbr_shader = PbrShaderProgram::new()?;

        Ok(Self {
            geometry,
            textures,
            pbr_shader,
        })
    }

    fn map_world_texture(
        world_texture: &dragonglass_world::Texture,
    ) -> dragonglass_opengl::Texture {
        let pixel_format = match world_texture.format {
            TextureFormat::R8 => gl::R8,
            TextureFormat::R8G8 => gl::RG,
            TextureFormat::R8G8B8 => gl::RGB,
            TextureFormat::R8G8B8A8 => gl::RGBA,
            TextureFormat::B8G8R8 => gl::BGR,
            TextureFormat::B8G8R8A8 => gl::BGRA,
            TextureFormat::R16 => gl::R16,
            TextureFormat::R16G16 => gl::RG16,
            TextureFormat::R16G16B16 => gl::RGB16,
            TextureFormat::R16G16B16A16 => gl::RGBA16,
            TextureFormat::R16F => gl::R16F,
            TextureFormat::R16G16F => gl::RG16F,
            TextureFormat::R16G16B16F => gl::RGB16F,
            TextureFormat::R16G16B16A16F => gl::RGBA16F,
            TextureFormat::R32 => gl::R32UI,
            TextureFormat::R32G32 => gl::RG32UI,
            TextureFormat::R32G32B32 => gl::RGB32UI,
            TextureFormat::R32G32B32A32 => gl::RGBA32UI,
            TextureFormat::R32F => gl::R32F,
            TextureFormat::R32G32F => gl::RG32F,
            TextureFormat::R32G32B32F => gl::RGB32F,
            TextureFormat::R32G32B32A32F => gl::RGBA32F,
        };

        let mut texture = Texture::new();
        texture.load_data(
            world_texture.width,
            world_texture.height,
            &world_texture.pixels,
            pixel_format,
        );
        texture
    }

    pub fn render(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        self.pbr_shader.update(world, aspect_ratio)?;
        self.geometry.bind();
        for alpha_mode in [AlphaMode::Opaque, AlphaMode::Mask, AlphaMode::Blend].iter() {
            for graph in world.scene.graphs.iter() {
                graph.walk(|node_index| {
                    let entity = graph[node_index];
                    let entry = world.ecs.entry_ref(entity)?;
                    let mesh_component = match entry.get_component::<MeshRender>() {
                        Ok(mesh_component) => mesh_component,
                        Err(_) => return Ok(()),
                    };
                    let mesh = match world.geometry.meshes.get(&mesh_component.name) {
                        Some(mesh) => mesh,
                        None => return Ok(()),
                    };
                    Self::set_blend_mode(alpha_mode);
                    let global_transform = world.global_transform(graph, node_index)?;
                    let model_matrix = world.entity_model_matrix(entity, global_transform)?;
                    self.pbr_shader.update_model_matrix(model_matrix);
                    self.render_mesh(mesh, world, alpha_mode)?;
                    Ok(())
                })?;
            }
        }

        Ok(())
    }

    fn set_blend_mode(alpha_mode: &AlphaMode) {
        match alpha_mode {
            AlphaMode::Opaque | AlphaMode::Mask => unsafe {
                gl::Disable(gl::BLEND);
            },
            AlphaMode::Blend => unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            },
        }
    }

    fn render_mesh(&self, mesh: &Mesh, world: &World, alpha_mode: &AlphaMode) -> Result<()> {
        for primitive in mesh.primitives.iter() {
            let material = match primitive.material_index {
                Some(material_index) => {
                    let primitive_material = world.material_at_index(material_index)?;
                    if primitive_material.alpha_mode != *alpha_mode {
                        return Ok(());
                    }
                    primitive_material.clone()
                }
                None => Material::default(),
            };

            self.pbr_shader.update_material(&material, &self.textures)?;

            let ptr: *const u8 = ptr::null_mut();
            let ptr = unsafe { ptr.add(primitive.first_index * std::mem::size_of::<u32>()) };
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    primitive.number_of_indices as _,
                    gl::UNSIGNED_INT,
                    ptr as *const _,
                );
            }
        }

        Ok(())
    }
}
