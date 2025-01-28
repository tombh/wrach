//! The code that manages the GPU compute workers

use bevy::{prelude::*, render::render_resource::BufferUsages};
use bevy_easy_compute::prelude::*;
use wrach_cpu_gpu_shared::PREFIX_SUM_OFFSET_HACK;

use crate::{
    compute::{
        buffers::Buffers,
        prefix_sum::{
            PrefixSumShaderBoxSumsAux, PrefixSumShaderBoxSumsMain, PrefixSumShaderDownSweepAux,
            PrefixSumShaderDownSweepMain,
        },
    },
    config_shader::ShaderWorldSettings,
    WrachState,
};

/// The main GPU compute pipeline for physics simulations
#[derive(Resource)]
pub struct PhysicsComputeWorker;

/// Just a convenient holder for cell counts
#[allow(clippy::missing_docs_in_private_items)]
struct CellCounts {
    main_u32: u32,
    main_usize: usize,
    aux_u32: u32,
    aux_usize: usize,
}

impl PhysicsComputeWorker {
    /// The size of the local shader workgroups for particle-related shaders. This is basically the
    /// number of threads that a single workgroup invocation will run. Apparently Nvidia has 32-width
    /// workgroups and AMD has 64, so I think a multiple of 64 is best to get full occupancy?
    pub const PARTICLE_WORKGROUP_LOCAL_SIZE: u32 = 64;

    /// Get meta data about the various cell counts.
    #[allow(clippy::expect_used)]
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::as_conversions)]
    #[allow(clippy::cast_possible_truncation)]
    fn get_cell_meta_data(state: &Mut<WrachState>) -> CellCounts {
        let (cells, _grid) = state.particle_store.spatial_bin.get_active_cells();
        let total_main_usize =
            cells.len() + Self::PREFIX_SUM_GUARD_ITEM + PREFIX_SUM_OFFSET_HACK as usize;
        let total_main_u32: u32 = total_main_usize
            .try_into()
            // Wow, imagine if we're simulating that many cells!
            .expect("Couldn't convert total main cells count into u32");

        let mut total_aux_u32 = state.particle_store.spatial_bin.calculate_total_aux_cells();
        total_aux_u32 += Self::PREFIX_SUM_GUARD_ITEM as u32 + PREFIX_SUM_OFFSET_HACK;
        let total_aux_usize = total_aux_u32 as usize;

        assert!(
            total_aux_u32 < Self::MAX_CELLS_FOR_PREFIX_SUM_PIPELINE,
            "More cells than our current 2-pass prefix sum pipeline can handle"
        );

        debug!(
            "Total spatial bins cells: main: {:?}, aux: {:?}",
            total_main_u32, total_aux_u32,
        );

        CellCounts {
            main_u32: total_main_u32,
            main_usize: total_main_usize,
            aux_u32: total_aux_u32,
            aux_usize: total_aux_usize,
        }
    }
}

impl ComputeWorker for PhysicsComputeWorker {
    #[expect(
        clippy::expect_used,
        reason = "`expect`s until there's a way to use `?` in systems"
    )]
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let mut state = world.resource_mut::<WrachState>();

        let cell_counts = Self::get_cell_meta_data(&state);
        let max_particles = state.particle_store.max_particles_per_frame();
        let max_particles_usize: usize = max_particles
            .try_into()
            .expect("Couldn't convert `max_particles` to `Vec` capacity");

        let indices = vec![0_u32; cell_counts.main_usize];
        let indices_aux = vec![0_u32; cell_counts.aux_usize];

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

        let config = state.config;

        let mut builder = AppComputeWorkerBuilder::new(world);
        builder
            .set_extra_buffer_usages(Some(BufferUsages::VERTEX))
            .add_uniform(Buffers::WORLD_SETTINGS_UNIFORM, &shader_settings)
            .set_extra_buffer_usages(None)
            // GPU-only
            .add_staging(Buffers::INDICES_AUX, &indices_aux)
            .add_storage(Buffers::INDICES_BLOCK_SUMS, &indices)
            .add_storage(Buffers::POSITIONS_IN, &positions)
            .add_storage(Buffers::VELOCITIES_IN, &velocities)
            .add_storage(Buffers::POSITIONS_AUX, &positions)
            .add_storage(Buffers::VELOCITIES_AUX, &velocities)
            // Readable from the CPU
            .add_staging(Buffers::INDICES_MAIN, &indices)
            .set_extra_buffer_usages(Some(BufferUsages::VERTEX))
            .add_staging(Buffers::POSITIONS_OUT, &positions)
            .set_extra_buffer_usages(None)
            .add_staging(Buffers::VELOCITIES_OUT, &velocities);

        builder = Self::particles_per_cell_count(builder, max_particles);
        builder = Self::prefix_sum::<PrefixSumShaderDownSweepMain, PrefixSumShaderBoxSumsMain>(
            builder,
            Buffers::INDICES_MAIN,
            cell_counts.main_u32,
        );
        builder = Self::prefix_sum::<PrefixSumShaderDownSweepAux, PrefixSumShaderBoxSumsAux>(
            builder,
            Buffers::INDICES_AUX,
            cell_counts.aux_u32,
        );
        builder = Self::particle_data(builder, max_particles);

        if !config.exclude_integration_pass {
            builder = Self::integration(builder, cell_counts.main_u32);
        }

        builder.build()
    }
}
