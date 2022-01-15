use dragonglass_dependencies::{env_logger, log::error};
use dragonglass_shader::compile_shaders;
use std::{boxed::Box, error::Error, fs::File};

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
    env_logger::init();
    if compile_shaders("../../assets/shaders/**/*.glsl").is_err() {
        error!("Failed to recompile shaders!");
    }
    Ok(())
}
