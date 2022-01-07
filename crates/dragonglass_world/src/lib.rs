mod gltf;
mod physics;
mod world;

pub use self::{gltf::*, physics::*, world::*};

use dragonglass_dependencies::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "dragonglass_dependencies::serde")]
pub struct Name(pub String);
