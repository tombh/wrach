//! With the generated prefix sum of the particles cell locations, pack the new particle data into
//! the "*_IN" buffers ready for the next frame of the simulation.

use bevy::reflect::TypePath;
use bevy_easy_compute::prelude::{AppComputeWorkerBuilder, ComputeShader, ShaderRef};

use super::{buffers::Buffers, PhysicsComputeWorker};

impl PhysicsComputeWorker {
    /// Count the number of particles per cell
    pub fn particle_data(
        mut builder: AppComputeWorkerBuilder<Self>,
        total_particles: u32,
    ) -> AppComputeWorkerBuilder<Self> {
        builder.add_pass::<PackNewParticleDataShader>(
            PackNewParticleDataShader::workgroups(total_particles),
            &[
                Buffers::WORLD_SETTINGS_UNIFORM,
                Buffers::INDICES_MAIN,
                Buffers::POSITIONS_OUT,
                Buffers::VELOCITIES_OUT,
                Buffers::POSITIONS_IN,
                Buffers::VELOCITIES_IN,
                Buffers::INDICES_AUX,
                Buffers::POSITIONS_AUX,
                Buffers::VELOCITIES_AUX,
            ],
        );
        builder
    }
}

/// Efficiently pack particles ready for the next frame
#[derive(TypePath)]
struct PackNewParticleDataShader;

impl PackNewParticleDataShader {
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
impl ComputeShader for PackNewParticleDataShader {
    fn shader() -> ShaderRef {
        "embedded://wrach_bevy/plugin/../../../../assets/shaders/pack_new_particle_data.wgsl".into()
    }

    fn entry_point<'shader>() -> &'shader str {
        "main"
    }
}

#[allow(clippy::default_numeric_fallback)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use bevy::math::Vec2;
    use bevy::math::Vec4;

    use crate::particle_store::ParticleStore;
    use crate::tests::utils::WrachTestAPI;
    use crate::Particle;
    use crate::WrachConfig;

    #[test]
    fn packed_data() {
        let dimensions = (10, 10);
        let cell_size = 3;

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
                position: Vec2::new(2.5, 2.5),
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

        wrach.tick_until_first_data();

        let gpu_packed_data = &wrach.get_simulation_state().packed_data;
        let cpu_packed_data = store.create_packed_data();

        assert_eq!(
            cpu_packed_data.positions,
            vec![
                Vec2::new(0.1, 0.1),
                Vec2::new(2.5, 2.5),
                Vec2::new(5.0, 5.0),
                Vec2::new(10.0, 10.0)
            ]
        );

        let mut allow_random_order_in_cell = gpu_packed_data.positions.clone();
        #[allow(clippy::min_ident_chars)]
        allow_random_order_in_cell[0..2].sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        assert_eq!(
            allow_random_order_in_cell[0..2],
            vec![Vec2::new(0.1, 0.1), Vec2::new(2.5, 2.5)]
        );

        assert_eq!(
            gpu_packed_data.positions[2..4],
            vec![Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0)]
        );
    }
}
