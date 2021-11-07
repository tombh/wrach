use spirv_builder::SpirvBuilder;
use std::error::Error;

fn build_shader(path_to_create: &str) -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new(path_to_create, "spirv-unknown-vulkan1.1")
        .print_metadata(spirv_builder::MetadataPrintout::Full)
        .build()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    build_shader("shaders")?;
    Ok(())
}
