//! Count the number of particles per spatial bin cell.
//!
//! Once Bevy upgrades wgpu/naga (probably in one of the Bevy 0.15 releases), we'll be able to do
//! this in the main integration shader with Rust/SPIRV atomics.

use bevy::reflect::TypePath;
use bevy_easy_compute::prelude::{AppComputeWorkerBuilder, ComputeShader, ShaderRef};

use super::PhysicsComputeWorker;

impl PhysicsComputeWorker {
    /// Count the number of particles per cell
    pub fn particles_per_cell_count(
        mut builder: AppComputeWorkerBuilder<Self>,
        total_particles: u32,
    ) -> AppComputeWorkerBuilder<Self> {
        builder.add_pass::<ParticlesPerCellCounterShader>(
            ParticlesPerCellCounterShader::workgroups(total_particles),
            &[
                Self::WORLD_SETTINGS_UNIFORM,
                Self::POSITIONS_BUFFER_OUT,
                Self::INDICES_BUFFER,
            ],
        );
        builder
    }
}

/// The shader for counting particles per cell. Can be included in the previous shader once Bevy
/// supports Naga's SPIRV atomics, [see:](https://github.com/gfx-rs/wgpu/issues/4489)
#[derive(TypePath)]
struct ParticlesPerCellCounterShader;

impl ParticlesPerCellCounterShader {
    /// Calculate workgroups
    const fn workgroups(total_particles: u32) -> [u32; 3] {
        [
            total_particles.div_ceil(PhysicsComputeWorker::PARTICLE_WORKGROUP_LOCAL_SIZE),
            1,
            1,
        ]
    }
}

#[allow(clippy::missing_trait_methods)]
impl ComputeShader for ParticlesPerCellCounterShader {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/../../../assets/shaders/particles_per_cell.wgsl".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "main"
    }
}
