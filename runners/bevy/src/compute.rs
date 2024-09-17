//! The code that manages the GPU compute workers

use bevy::{prelude::*, reflect::TypePath};
use bevy_easy_compute::prelude::*;

use crate::{config_shader::ShaderWorldConfig, WrachState};

/// The main GPU compute pipeline for physics simulations
#[derive(Resource)]
pub struct PhysicsComputeWorker;

impl PhysicsComputeWorker {
    /// Config data for the simulation
    pub const WORLD_CONFIG_UNIFORM: &'static str = "world_config";
    /// Efficient packing of particle indices and spatial bin cell counts
    pub const INDICES_BUFFER_IN: &'static str = "indices_in";
    /// See above. Particles indices for writing.
    pub const INDICES_BUFFER_OUT: &'static str = "indices_out";
    /// Pixel positions buffer ID for reading
    pub const POSITIONS_BUFFER_IN: &'static str = "positions_in";
    /// Pixel positions buffer ID for writing
    pub const POSITIONS_BUFFER_OUT: &'static str = "positions_out";
    /// Pixel velocities buffer ID for reading
    pub const VELOCITIES_BUFFER_IN: &'static str = "velocities_in";
    /// Pixel velocities buffer ID for writing
    pub const VELOCITIES_BUFFER_OUT: &'static str = "velocities_out";
}

impl ComputeWorker for PhysicsComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let state = world.resource::<WrachState>();

        let total_cells_usize = state.particle_store.spatial_bin.get_active_cells().len();
        #[allow(clippy::expect_used)]
        let total_cells: u32 = total_cells_usize
            .try_into()
            .expect("Couldn't convert total cells count into u32");

        #[allow(clippy::expect_used)]
        let max_particles: usize = state
            .particle_store
            .max_particles_per_frame()
            .try_into()
            .expect("Couldn't convert `max_particles` to `Vec` capacity");

        #[allow(clippy::arithmetic_side_effects)]
        let indices = vec![0_u32; total_cells_usize + 1];

        let positions = vec![Vec2::default(); max_particles];
        let velocities = vec![Vec2::default(); max_particles];

        // TODO: Explain and explore workgroup sizes
        let partition = 8;
        let main_workgroup_size = u32::div_ceil(total_cells, partition);
        let workgroups = [main_workgroup_size, partition, 1];

        let wrach_world_config = ShaderWorldConfig {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::as_conversions)]
            view_dimensions: Vec2::new(
                state.config.dimensions.0.into(),
                state.config.dimensions.1.into(),
            ),
            view_anchor: Vec2::new(0.0, 0.0),
        };

        AppComputeWorkerBuilder::new(world)
            .add_uniform(Self::WORLD_CONFIG_UNIFORM, &wrach_world_config)
            .add_staging(Self::INDICES_BUFFER_IN, &indices)
            .add_staging(Self::INDICES_BUFFER_OUT, &indices)
            .add_staging(Self::POSITIONS_BUFFER_IN, &positions)
            .add_staging(Self::POSITIONS_BUFFER_OUT, &positions)
            .add_staging(Self::VELOCITIES_BUFFER_IN, &velocities)
            .add_staging(Self::VELOCITIES_BUFFER_OUT, &velocities)
            .add_pass::<FirstPassShader>(
                workgroups,
                &[
                    Self::WORLD_CONFIG_UNIFORM,
                    Self::INDICES_BUFFER_IN,
                    Self::INDICES_BUFFER_OUT,
                    Self::POSITIONS_BUFFER_IN,
                    Self::POSITIONS_BUFFER_OUT,
                    Self::VELOCITIES_BUFFER_IN,
                    Self::VELOCITIES_BUFFER_OUT,
                ],
            )
            // TODO: Hack. Re-add once we have GPU-side prefix sums
            // .add_swap(Self::INDICES_BUFFER_IN, Self::INDICES_BUFFER_OUT)
            // .add_swap(Self::POSITIONS_BUFFER_IN, Self::POSITIONS_BUFFER_OUT)
            // .add_swap(Self::VELOCITIES_BUFFER_IN, Self::VELOCITIES_BUFFER_OUT)
            .build()
    }
}

/// The shader for the first pass
#[derive(TypePath)]
struct FirstPassShader;

#[allow(clippy::missing_trait_methods)]
impl ComputeShader for FirstPassShader {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/../../../assets/shaders/wrach_physics.spv".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "main"
    }
}
