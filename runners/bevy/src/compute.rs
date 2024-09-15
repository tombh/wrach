//! The code that manages the GPU compute workers

use bevy::{prelude::*, reflect::TypePath};
use bevy_easy_compute::prelude::*;

use crate::{config_shader::ShaderWorldConfig, WrachState};

/// The main GPU compute pipeline for physics simulations
#[derive(Resource)]
pub struct PhysicsComputeWorker;

impl PhysicsComputeWorker {
    /// Pixel positions buffer ID for reading
    pub const POSITIONS_BUFFER_IN: &'static str = "positions_in";
    /// Pixel positions buffer ID for writing
    pub const POSITIONS_BUFFER_OUT: &'static str = "positions_out";
    /// Pixel velocities buffer ID for reading
    pub const VELOCITIES_BUFFER_IN: &'static str = "velocities_in";
    /// Pixel velocities buffer ID for writing
    pub const VELOCITIES_BUFFER_OUT: &'static str = "velocities_out";
    /// Config data for the simulation
    pub const WORLD_CONFIG_UNIFORM: &'static str = "world_config";
}

impl ComputeWorker for PhysicsComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let state = world.resource::<WrachState>();

        #[allow(clippy::expect_used)]
        let max_particles: usize = state
            .max_particles
            .try_into()
            .expect("Couldn't convert `max_particles` to `Vec` capacity");

        let positions = vec![Vec2::default(); max_particles];
        let velocities = vec![Vec2::default(); max_particles];

        // TODO: Explain and explore workgroup sizes
        let partition = 8;
        let main_workgroup_size = u32::div_ceil(state.max_particles, partition);
        let workgroups = [main_workgroup_size, partition, 1];

        let wrach_world_config = ShaderWorldConfig {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::as_conversions)]
            dimensions: Vec2::new(
                state.config.dimensions.0 as f32,
                state.config.dimensions.1 as f32,
            ),
            view_anchor: Vec2::new(0.0, 0.0),
        };

        AppComputeWorkerBuilder::new(world)
            .add_uniform(Self::WORLD_CONFIG_UNIFORM, &wrach_world_config)
            .add_staging(Self::POSITIONS_BUFFER_IN, &positions)
            .add_staging(Self::POSITIONS_BUFFER_OUT, &positions)
            .add_staging(Self::VELOCITIES_BUFFER_IN, &velocities)
            .add_staging(Self::VELOCITIES_BUFFER_OUT, &velocities)
            .add_pass::<FirstPassShader>(
                workgroups,
                &[
                    Self::WORLD_CONFIG_UNIFORM,
                    Self::POSITIONS_BUFFER_IN,
                    Self::POSITIONS_BUFFER_OUT,
                    Self::VELOCITIES_BUFFER_IN,
                    Self::VELOCITIES_BUFFER_OUT,
                ],
            )
            .add_swap(Self::POSITIONS_BUFFER_IN, Self::POSITIONS_BUFFER_OUT)
            .add_swap(Self::VELOCITIES_BUFFER_IN, Self::VELOCITIES_BUFFER_OUT)
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
