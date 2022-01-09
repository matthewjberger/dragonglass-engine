mod gltf;
mod physics;
mod world;

pub use self::{gltf::*, physics::*, world::*};

pub use dragonglass_dependencies::legion::EntityStore;

#[derive(Serialize, Deserialize)]
#[serde(crate = "dragonglass_dependencies::serde")]
pub struct Name(pub String);

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "dragonglass_dependencies::serde")]
pub struct Selected;
