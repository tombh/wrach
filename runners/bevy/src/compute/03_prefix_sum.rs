//! The Prefix Sum algorithm computes a running sum of elements in an array.
//! It works by iterating through the array, maintaining a cumulative sum.
//!
//! For example:
//! Input:  [0, 1, 0, 3, 2]
//! Output: [0, 1, 1, 4, 6]
//!
//! In particle simulations it's used for being able to efficiently lookup nearby particles through
//! spatial bins, or cells, as we call them here. A cell contains exponentially fewer particles and
//! can also be easily looked up based on the particle you're finding neighbours for.
//!
//! The prefix sum is a 1-dimensional list of all the cells in the current viewport frame. Each
//! item in the prefix sum is an index pointing to the first particle in the cell. The next item
//! then can be used to calculate the number of particles in the cell because it points to the
//! index of the start of the first particle in the next cell. The particle data is packed
//! efficiently such that all particle data for a cell is stored together. This also contributes
//! towards reducing bank conflicts when looking up the particle data.

use bevy::reflect::TypePath;
use bevy_easy_compute::prelude::{AppComputeWorkerBuilder, ComputeShader, ShaderRef};

use super::{buffers::Buffers, PhysicsComputeWorker};

impl PhysicsComputeWorker {
    /// Our current prefix sum implementation only has 2 passes, we can easily add more passes, but I
    /// don't think we'll ever need that many cells (ie prefix sum items). Besides, hopefully we'll
    /// move to a "sub group"-based algorithm soon which is both faster and doesn't require multiple
    /// passes. I suspect that will force a dependency on Vulkan though.
    pub const MAX_CELLS_FOR_PREFIX_SUM_PIPELINE: u32 = 4_194_304;

    /// This is calculated by:
    ///   `local.workgroup_size.x * local.workgroup_size.y * 2`
    /// See prefix sum shader for hardcoded workgroup sizes.
    pub const PREFIX_SUM_ITEMS_PER_WORKGROUP: u32 = 2048;

    /// An extra "guard item" at then end, so we can store the final cell's particle count.
    /// So if the final cell has no particles, instead of: [0, 3, 3, 4], we do: [0, 3, 3, 4, 4].
    /// And if the final cell has 1 particle, instead of: [0, 3, 3, 4], we do: [0, 3, 3, 4, 5].
    pub const PREFIX_SUM_GUARD_ITEM: usize = 1;

    // TODO: Move to a shared CPU/GPU crate.
    /// Our current GPU implementation of prefix sum, offsets all items one to the right. Hopefully
    /// we'll move to a subgroup-based version soon that doesn't have this issue.
    pub const PREFIX_SUM_OFFSET_HACK: usize = 1;

    /// Count the number of particles per cell
    pub fn prefix_sum(
        mut builder: AppComputeWorkerBuilder<Self>,
        total_cells: u32,
    ) -> AppComputeWorkerBuilder<Self> {
        builder.add_pass::<PrefixSumShaderDownSweep>(
            PrefixSumShaderDownSweep::workgroups(total_cells),
            &[
                Buffers::WORLD_SETTINGS_UNIFORM,
                Buffers::INDICES_MAIN,
                Buffers::INDICES_BLOCK_SUMS,
            ],
        );

        let remaining = total_cells.div_ceil(Self::PREFIX_SUM_ITEMS_PER_WORKGROUP);
        if remaining > 1 {
            builder
                .add_pass::<PrefixSumShaderDownSweep>(
                    PrefixSumShaderDownSweep::workgroups(remaining),
                    &[
                        Buffers::WORLD_SETTINGS_UNIFORM,
                        // Note how, this time, the block sums are in place of the indices.
                        Buffers::INDICES_BLOCK_SUMS,
                        Buffers::INDICES_BLOCK_SUMS,
                    ],
                )
                .add_pass::<PrefixSumShaderBoxSums>(
                    PrefixSumShaderBoxSums::workgroups(total_cells),
                    &[
                        Buffers::WORLD_SETTINGS_UNIFORM,
                        Buffers::INDICES_MAIN,
                        Buffers::INDICES_BLOCK_SUMS,
                    ],
                );
        };

        builder
    }
}

/// First stage of a 2-stage prefix sum algorithm.
#[derive(TypePath)]
struct PrefixSumShaderDownSweep;

impl PrefixSumShaderDownSweep {
    /// Calculate workgroups
    const fn workgroups(total_cells: u32) -> [u32; 3] {
        let main_workgroup_size = u32::div_ceil(
            total_cells,
            PhysicsComputeWorker::PREFIX_SUM_ITEMS_PER_WORKGROUP,
        );
        [main_workgroup_size, 1, 1]
    }
}

#[allow(clippy::missing_trait_methods)]
impl ComputeShader for PrefixSumShaderDownSweep {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/plugin/../../../../assets/shaders/prefix_sum.wgsl".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "reduce_downsweep"
    }
}

/// Second stage of a 2-stage prefix sum algorithm.
#[derive(TypePath)]
struct PrefixSumShaderBoxSums;

impl PrefixSumShaderBoxSums {
    /// Calculate workgroups
    const fn workgroups(total_cells: u32) -> [u32; 3] {
        let main_workgroup_size = u32::div_ceil(
            total_cells,
            PhysicsComputeWorker::PREFIX_SUM_ITEMS_PER_WORKGROUP,
        );
        [main_workgroup_size, 1, 1]
    }
}

#[allow(clippy::missing_trait_methods)]
impl ComputeShader for PrefixSumShaderBoxSums {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/plugin/../../../../assets/shaders/prefix_sum.wgsl".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "add_block_sums"
    }
}

#[allow(clippy::default_numeric_fallback)]
#[cfg(test)]
mod test {
    use bevy::math::Vec2;
    use bevy::math::Vec4;

    use crate::particle_store::ParticleStore;
    use crate::tests::utils::WrachTestAPI;
    use crate::Particle;
    use crate::WrachConfig;

    #[test]
    fn prefix_sum_for_small_arrays() {
        let dimensions = (10, 10);
        let cell_size = 5;

        let mut wrach = WrachTestAPI::new(WrachConfig {
            dimensions,
            cell_size,
            ..Default::default()
        });
        let mut store = ParticleStore::new(
            cell_size,
            Vec4::new(0.0, 0.0, dimensions.0.into(), dimensions.1.into()),
        );

        let particles = vec![
            Particle {
                position: Vec2::new(0.1, 0.1),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(2.0, 2.0),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(f32::from(dimensions.0) / 2.0, f32::from(dimensions.1) / 2.0),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(dimensions.0.into(), dimensions.1.into()),
                velocity: Vec2::new(0.0, 0.0),
            },
        ];

        wrach.add_particles(particles.clone());
        for particle in particles {
            store.add_particle(particle);
        }

        for _ in 0..4 {
            wrach.tick();
        }

        let gpu_packed_data = &wrach.get_simulation_state().packed_data;
        let cpu_packed_data = store.create_packed_data();

        assert_eq!(
            cpu_packed_data.indices,
            vec![0, 0, 2, 2, 2, 2, 3, 3, 3, 3, 4]
        );

        //assert_eq!(gpu_packed_data.positions, cpu_packed_data.positions);

        assert_eq!(
            gpu_packed_data.indices, cpu_packed_data.indices,
            "GPU packed indices do not match CPU packed indices"
        );
    }

    #[test]
    fn prefix_sum_for_large_arrays() {
        let dimensions = (500, 300);
        let cell_size = 6;

        let mut wrach = WrachTestAPI::new(WrachConfig {
            dimensions,
            cell_size,
            ..Default::default()
        });
        let mut store = ParticleStore::new(
            cell_size,
            Vec4::new(0.0, 0.0, dimensions.0.into(), dimensions.1.into()),
        );

        let particles = vec![
            Particle {
                position: Vec2::new(1.1, 1.1),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(2.2, 2.2),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(f32::from(dimensions.0) / 2.0, f32::from(dimensions.1) / 2.0),
                velocity: Vec2::new(0.0, 0.0),
            },
            Particle {
                position: Vec2::new(dimensions.0.into(), dimensions.1.into()),
                velocity: Vec2::new(0.0, 0.0),
            },
        ];

        wrach.add_particles(particles.clone());
        for particle in particles {
            store.add_particle(particle);
        }

        for _ in 0..4 {
            wrach.tick();
        }

        let gpu_packed_data = &wrach.get_simulation_state().packed_data;
        let cpu_packed_data = store.create_packed_data();

        assert_eq!(gpu_packed_data.indices.len(), cpu_packed_data.indices.len());
        assert_eq!(
            gpu_packed_data.indices, cpu_packed_data.indices,
            "GPU packed indices do not match CPU packed indices"
        );
    }
}
