use crate::opengl::world::Light;
use dragonglass_dependencies::{
    anyhow::{bail, Result},
    nalgebra_glm as glm,
};
use dragonglass_opengl::{ShaderProgram, Texture};
use dragonglass_world::{Material, World};

pub struct PbrShader {
    pub shader_program: ShaderProgram,
}

impl PbrShader {
    pub fn new() -> Result<Self> {
        let mut shader_program = ShaderProgram::new();
        shader_program
            .vertex_shader_source(VERTEX_SHADER_SOURCE)?
            .fragment_shader_source(FRAGMENT_SHADER_SOURCE)?
            .link();
        Ok(Self { shader_program })
    }

    pub fn update(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        self.shader_program.use_program();
        self.upload_lights(world)?;
        self.update_uniforms(world, aspect_ratio)?;
        Ok(())
    }

    fn upload_lights(&self, world: &World) -> Result<()> {
        let world_lights = world
            .lights()?
            .iter()
            .map(|(transform, light)| Light::from_node(transform, light))
            .collect::<Vec<_>>();
        for (index, light) in world_lights.iter().enumerate() {
            let name = |key: &str| format!("lights[{}].{}", index, key);
            self.shader_program
                .set_uniform_vec3(&name("direction"), light.direction.as_slice());
            self.shader_program
                .set_uniform_float(&name("range"), light.range);
            self.shader_program
                .set_uniform_vec3(&name("color"), light.color.as_slice());
            self.shader_program
                .set_uniform_float(&name("intensity"), light.intensity);
            self.shader_program
                .set_uniform_vec3(&name("position"), light.position.as_slice());
            self.shader_program
                .set_uniform_float(&name("innerConeCos"), light.inner_cone_cos);
            self.shader_program
                .set_uniform_float(&name("outerConeCos"), light.outer_cone_cos);
            self.shader_program
                .set_uniform_int(&name("kind"), light.kind);
        }
        self.shader_program
            .set_uniform_int("numberOfLights", world_lights.len() as _);
        Ok(())
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

    pub fn update_model_matrix(&self, model_matrix: glm::Mat4) {
        self.shader_program
            .set_uniform_matrix4x4("model", model_matrix.as_slice());
    }

    pub fn update_material(&self, material: &Material, textures: &[Texture]) -> Result<()> {
        self.shader_program.set_uniform_vec4(
            "material.baseColorFactor",
            material.base_color_factor.as_slice(),
        );

        self.shader_program.set_uniform_vec4(
            "material.emissiveFactor",
            glm::vec3_to_vec4(&material.emissive_factor).as_slice(),
        );

        self.shader_program
            .set_uniform_int("material.alphaMode", material.alpha_mode as _);

        self.shader_program
            .set_uniform_float("material.alphaCutoff", material.alpha_cutoff);

        self.shader_program
            .set_uniform_float("material.occlusionStrength", material.occlusion_strength);

        self.shader_program
            .set_uniform_float("material.metallicFactor", material.metallic_factor);

        self.shader_program
            .set_uniform_float("material.roughnessFactor", material.roughness_factor);

        for (index, descriptor) in ["Diffuse", "Physical", "Normal", "Occlusion", "Emissive"]
            .iter()
            .enumerate()
        {
            let texture_index = match *descriptor {
                "Diffuse" => material.color_texture_index,
                "Physical" => material.metallic_roughness_texture_index,
                "Normal" => material.normal_texture_index,
                "Occlusion" => material.occlusion_texture_index,
                "Emissive" => material.emissive_texture_index,
                _ => bail!("Failed to find index for texture type!"),
            };
            let has_texture = texture_index > -1;

            self.shader_program
                .set_uniform_bool(&format!("material.has{}Texture", *descriptor), has_texture);

            self.shader_program
                .set_uniform_int(&format!("{}Texture", *descriptor), index as _);

            if has_texture {
                textures[texture_index as usize].bind(index as _);
            }
        }

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

out vec3 Position;
out vec2 UV0;
out vec3 Normal;
out vec3 Color0;

void main()
{
   Position = vec3(model * vec4(inPosition, 1.0));
   gl_Position = projection * view * vec4(Position, 1.0);
   UV0 = inUV0;
   Normal = mat3(model) * inNormal;
   Color0 = inColor0;
}
"#;

const FRAGMENT_SHADER_SOURCE: &'static str = &r#"
#version 450 core
struct Light
{
    vec3 direction;
    float range;
    vec3 color;
    float intensity;
    vec3 position;
    float innerConeCos;
    float outerConeCos;
    int kind;
    vec2 padding;
};
#define MAX_NUMBER_OF_LIGHTS 4
uniform Light lights[MAX_NUMBER_OF_LIGHTS];
uniform int numberOfLights;
struct Material {
    vec4 baseColorFactor;
    vec4 emissiveFactor;
    int alphaMode;
    float alphaCutoff;
    float occlusionStrength;
    float metallicFactor;
    float roughnessFactor;
    bool isUnlit;
    bool hasDiffuseTexture;
    bool hasPhysicalTexture;
    bool hasNormalTexture;
    bool hasOcclusionTexture;
    bool hasEmissiveTexture;
}; 
uniform Material material;
uniform sampler2D DiffuseTexture;
uniform sampler2D PhysicalTexture;
uniform sampler2D NormalTexture;
uniform sampler2D OcclusionTexture;
uniform sampler2D EmissiveTexture;
uniform vec3 cameraPosition;
in vec3 Position;
in vec2 UV0;
in vec3 Normal;
in vec3 Color0;
out vec4 color;
vec4 srgb_to_linear(vec4 srgbIn)
{
    return vec4(pow(srgbIn.xyz,vec3(2.2)),srgbIn.w);
}
const float PI = 3.14159265359;
vec3 getNormal();
float DistributionGGX(vec3 N, vec3 H, float roughness);
float GeometrySchlickGGX(float NdotV, float roughness);
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness);
vec3 fresnelSchlick(float cosTheta, vec3 F0);
void main(void)
{
    color = material.baseColorFactor;
    if (material.hasDiffuseTexture) {
        vec4 albedoMap = texture(DiffuseTexture, UV0);
        color = srgb_to_linear(albedoMap);
    }
    color *= vec4(Color0, 1.0);
    // alpha discard
    if (material.alphaMode == 2 && color.a < material.alphaCutoff) {
        discard;
    }
    if (material.isUnlit) {
        color = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
        return;
    }
    float metallic = material.metallicFactor;
    float roughness = material.roughnessFactor;
    if (material.hasPhysicalTexture)
    {
        vec4 physicalDescriptor = texture(PhysicalTexture, UV0);
        roughness *= physicalDescriptor.g;
        metallic *= physicalDescriptor.b;
    }
    // calculate reflectance at normal incidence; if dia-electric (like plastic) use F0 
    // of 0.04 and if it's a metal, use the albedo color as F0 (metallic workflow)    
    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, color.rgb, metallic);
    vec3 N = getNormal();
    vec3 V = normalize(cameraPosition - Position);
    vec3 R = reflect(-V, N); 
    // reflectance equation
    vec3 Lo = vec3(0.0);
    for(int i = 0; i < numberOfLights; ++i) 
    {
        Light light = lights[i];
        vec3 L = normalize(light.position - Position);
        vec3 H = normalize(V + L);
        float distance = length(light.position - Position);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light.color * attenuation;
        float NDF = DistributionGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);
        vec3 nominator = NDF * G * F;
        float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001;
        vec3 specular = nominator / denominator;
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;
        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * color.rgb / PI + specular) * radiance * NdotL;
    }
    color *= vec4(0.03);
    color += vec4(Lo, 0.0);
    float occlusion = 1.0;
    if (material.hasOcclusionTexture) {
         occlusion = texture(OcclusionTexture, UV0).r;
    }
    color = mix(color, color * occlusion, material.occlusionStrength);
    vec4 emission = vec4(0.0);
    if (material.hasEmissiveTexture) {
        emission = srgb_to_linear(texture(EmissiveTexture, UV0)) * vec4(material.emissiveFactor.rgb, 1.0);
    }
    color += vec4(emission.rgb, 0.0);
    // HDR tonemapping
    color = color / (color + vec4(1.0));
    // gamma correct
    color = pow(color, vec4(1.0/2.2));
}
vec3 getNormal()
{
    if (!material.hasNormalTexture) {
        return Normal;
    }
    vec3 tangentNormal = texture(NormalTexture, UV0).xyz * 2.0 - 1.0;
    vec3 Q1  = dFdx(Position);
    vec3 Q2  = dFdy(Position);
    vec2 st1 = dFdx(UV0);
    vec2 st2 = dFdy(UV0);
    vec3 N   = normalize(Normal);
    vec3 T  = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);
    return normalize(TBN * tangentNormal);
}
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    return nom / denom;
}
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;
    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    return nom / denom;
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}
"#;
