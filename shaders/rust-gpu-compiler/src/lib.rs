use spirv_builder::SpirvBuilder;

pub fn build(path_to_create: &str) -> Vec<u8> {
    println!("Compiling {} to SPIRV...", path_to_create);
    let compile_result = SpirvBuilder::new(path_to_create, "spirv-unknown-vulkan1.1")
        .print_metadata(spirv_builder::MetadataPrintout::None)
        // My Vulkan version and Intel Iris GPU seem to support this extension
        // and the SPIRV module compiles when using `spirv_std::macros::debug_printfln`
        // but when running I get:
        //   UnsupportedExtension("SPV_KHR_non_semantic_info")
        // Is it something to do with wgpu?
        // ---
        // .extension("SPV_KHR_non_semantic_info")
        // .extension("SPV_KHR_16bit_storage")
        // .capability(spirv_builder::Capability::Int16)
        .build()
        .unwrap();
    let module_path = compile_result.module.unwrap_single();
    std::fs::read(module_path).unwrap()
}
