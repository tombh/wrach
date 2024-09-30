//! Where the bulk of the main physics is done.

use bevy::reflect::TypePath;
use bevy_easy_compute::prelude::{AppComputeWorkerBuilder, ComputeShader, ShaderRef};

use super::{buffers::Buffers, PhysicsComputeWorker};

impl PhysicsComputeWorker {
    /// Shader for integrating the physics results onto the movement of particles
    pub fn integration(
        mut builder: AppComputeWorkerBuilder<Self>,
        total_cells: u32,
    ) -> AppComputeWorkerBuilder<Self> {
        builder.add_pass::<IntegrationShader>(
            IntegrationShader::workgroups(total_cells),
            &[
                Buffers::WORLD_SETTINGS_UNIFORM,
                Buffers::INDICES_MAIN,
                Buffers::POSITIONS_IN,
                Buffers::POSITIONS_OUT,
                Buffers::VELOCITIES_IN,
                Buffers::VELOCITIES_OUT,
                Buffers::INDICES_AUX,
                Buffers::POSITIONS_AUX,
                Buffers::VELOCITIES_AUX,
            ],
        );
        builder
    }
}

/// The shader for the first pass
#[derive(TypePath)]
struct IntegrationShader;

impl IntegrationShader {
    // TODO: Explain and explore workgroup sizes
    /// Calculate workgroup sizes
    const fn workgroups(total_cells: u32) -> [u32; 3] {
        let partition = 64;
        let main_workgroup_size = u32::div_ceil(total_cells, partition);
        [main_workgroup_size, 1, 1]
    }
}

#[allow(clippy::missing_trait_methods)]
impl ComputeShader for IntegrationShader {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/plugin/../../../../assets/shaders/wrach_physics.spv".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "main"
    }
}
