//! The code that manages the GPU compute workers

use bevy::{prelude::*, render::render_resource::BufferUsages};
use bevy_easy_compute::prelude::*;

use crate::{compute::buffers::Buffers, config_shader::ShaderWorldSettings, WrachState};

/// The main GPU compute pipeline for physics simulations
#[derive(Resource)]
pub struct PhysicsComputeWorker;

impl PhysicsComputeWorker {
    /// The size of the local shader workgroups for particle-related shaders. This is basically the
    /// number of threads that a single workgroup invocation will run. Apparently Nvidia has 32-width
    /// workgroups and AMD has 64, so I think a multiple of 64 is best to get full occupancy?
    pub const PARTICLE_WORKGROUP_LOCAL_SIZE: u32 = 64;
}

impl ComputeWorker for PhysicsComputeWorker {
    #[expect(
        clippy::expect_used,
        reason = "`expect`s until there's a way to use `?` in systems"
    )]
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let mut state = world.resource_mut::<WrachState>();

        let (cells, _grid) = state.particle_store.spatial_bin.get_active_cells();
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "Overflow and division by zero are unlikely"
        )]
        let total_cells_usize =
            cells.len() + Self::PREFIX_SUM_GUARD_ITEM + Self::PREFIX_SUM_OFFSET_HACK;
        let total_cells: u32 = total_cells_usize
            .try_into()
            // Wow, imagine if we're simulating that many cells!
            .expect("Couldn't convert total cells count into u32");

        assert!(
            total_cells < Self::MAX_CELLS_FOR_PREFIX_SUM_PIPELINE,
            "More cells than our current 2-pass prefix sum pipeline can handle"
        );

        debug!("Total spatial bins cells: {:?}", total_cells);

        let max_particles = state.particle_store.max_particles_per_frame();
        let max_particles_usize: usize = max_particles
            .try_into()
            .expect("Couldn't convert `max_particles` to `Vec` capacity");

        let indices = vec![0_u32; total_cells_usize];

        let positions = vec![Vec2::default(); max_particles_usize];
        let velocities = vec![Vec2::default(); max_particles_usize];

        let shader_settings = ShaderWorldSettings {
            view_dimensions: Vec2::new(
                state.config.dimensions.0.into(),
                state.config.dimensions.1.into(),
            ),
            view_anchor: Vec2::new(0.0, 0.0),
            grid_dimensions: state.particle_store.spatial_bin.grid_dimensions,
            cell_size: state.config.cell_size.into(),
            particles_in_frame_count: 0,
        };
        state.shader_settings = shader_settings;

        info!("{:?}", shader_settings);

        let mut builder = AppComputeWorkerBuilder::new(world);
        builder
            .set_extra_buffer_usages(Some(BufferUsages::VERTEX))
            .add_uniform(Buffers::WORLD_SETTINGS_UNIFORM, &shader_settings)
            .set_extra_buffer_usages(None)
            // GPU-only
            .add_storage(Buffers::INDICES_BLOCK_SUMS, &indices)
            .add_storage(Buffers::POSITIONS_OUT, &positions)
            .add_storage(Buffers::VELOCITIES_OUT, &velocities)
            // Readable from the CPU
            .add_staging(Buffers::INDICES_MAIN, &indices)
            .set_extra_buffer_usages(Some(BufferUsages::VERTEX))
            .add_staging(Buffers::POSITIONS_IN, &positions)
            .set_extra_buffer_usages(None)
            .add_staging(Buffers::VELOCITIES_IN, &velocities);

        builder = Self::integration(builder, total_cells);
        builder = Self::particles_per_cell_count(builder, max_particles);
        builder = Self::prefix_sum(builder, total_cells);
        builder = Self::particle_data(builder, max_particles);

        builder.build()
    }
}
