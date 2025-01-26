//! A hash store for particles

use bevy::{
    math::{Vec2, Vec4},
    utils::hashbrown::HashMap,
};

use crate::{
    spatial_bin::{PackedData, SpatialBin, SpatialBinCoord},
    Particle,
};

/// Store of all active particle data. Keyed by Spatial Binning coordinates
pub struct ParticleStore {
    /// A fast `HashMap` implementation from Bevy's Hashbrown
    pub hashmap: HashMap<SpatialBinCoord, ParticleData>,
    /// An instance of a `SpatialBin` that manages an efficient representation of the particles.
    pub spatial_bin: SpatialBin,
    /// Total number of particles simulated in this frame. This will normally be much smaller than
    /// the total number of particles that we have a record of.
    pub particles_in_frame_count: u32,
    /// The list of cells that are currently in the GPU and need to be read back into the store.
    /// This is needed because the user may change the viewport at any time, perhaps multiple times
    /// before the next frame is run. And the GPU only returns indices relative to the frame, not
    /// gloval cell coordinates.
    pub cells_to_read_from_gpu: Vec<SpatialBinCoord>,
}

/// Format of particle data to be stored in the store. This is the same format as it is used on the
/// GPU. Separating the fields into vectors allows compute and render stages to only read the data
/// they need. IO is expensive on GPUs.
#[derive(Default)]
pub struct ParticleData {
    /// Vector of particle positions
    pub positions: Vec<Vec2>,
    /// Vector of particle velocities
    pub velocities: Vec<Vec2>,
}

impl ParticleStore {
    /// Instantiate. `cell_size` is the size of a cell in the spatial bin
    pub fn new(cell_size: u16, viewport: Vec4) -> Self {
        let spatial_bin = SpatialBin::new(cell_size, viewport);
        Self {
            spatial_bin,
            hashmap: HashMap::new(),
            particles_in_frame_count: 0,
            cells_to_read_from_gpu: Vec::default(),
        }
    }

    /// Add a particle into the store. It will be placed into the spatial bin cell calculated from
    /// its position.
    pub fn add_particle(&mut self, particle: Particle) {
        let cell_coord = self.spatial_bin.get_cell_coord(particle.position);
        let entry = self.hashmap.entry(cell_coord).or_default();
        entry.positions.push(particle.position);
        entry.velocities.push(particle.velocity);
    }

    /// Add particles to the store. Overwrites previous cell.
    pub fn add_particles_to_cell(&mut self, cell: SpatialBinCoord, particles: ParticleData) {
        self.hashmap.insert(cell, particles);
    }

    /// Remove particles remove the store.
    pub fn remove(&mut self, cell: SpatialBinCoord) {
        self.hashmap.remove(&cell);
    }

    /// Create an efficient spatial representation of all the currently active particles in and
    /// around the viewport.
    ///
    // TODO:
    //   - Save GPU data to CPU memory and disk.
    //   - Probably use Persist with https://github.com/cberner/redb
    /// Take `PackedData` from the GPU and write back into the store.
    // pub fn update_from_gpu(&mut self, update: &PackedData) {
    // TODO: Once we have prefix sum working on the GPU
    // for (i, cell) in self.cells_to_read_from_gpu.iter().enumerate() {
    //     let particles_begin_at = update.indices[i];
    //     let marker = update.indices[i + 1];
    //     let particles_count = particles
    //     let particles
    // }
    #[expect(
        clippy::expect_used,
        reason = "`expect`s until there's a way to use `?` in systems"
    )]
    pub fn create_packed_data(&mut self) -> PackedData {
        let data = self.spatial_bin.create_packed_data(self);

        // TODO: move this to `update_from_gpu` once GPU prefix sum is fully working.
        {
            self.particles_in_frame_count = data
                .positions
                .len()
                .try_into()
                .expect("More particles than fit into u32");
        };

        data
    }

    /// Calculate the maximum number of particles involved in a single frame. Equal to
    /// those that can be seen from the viewport and those that make up a border of spatial bin
    /// cells around the viewport
    #[expect(
        clippy::expect_used,
        reason = "`expect`s until there's a way to use `?` in systems"
    )]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "I'm assuming that because of the numbers involved, there won't be any overflow"
    )]
    pub fn max_particles_per_frame(&self) -> u32 {
        // Extra particles to add.
        // TODO:
        //   We probably won't need this when we start using Bevy's dynamically-sized buffers.
        //   See: https://github.com/AnthonyTornetta/bevy_easy_compute/issues/14
        let extra_percent = 10;

        let (cells, _grid) = self.spatial_bin.get_active_cells();
        let total_cells: u32 = cells
            .len()
            .try_into()
            .expect("Couldn't convert cell count into u32");
        let particles_per_cell: u32 = self.spatial_bin.cell_size.pow(2).into();
        let total_particles_normally = total_cells * particles_per_cell;
        let one_percent = total_particles_normally.div_ceil(100);
        let extra_particles = extra_percent * one_percent;
        total_particles_normally + extra_particles
    }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing, reason = "Tests aren't so strict")]
mod tests {
    use bevy::math::{Vec2, Vec4};

    use super::*;

    #[test]
    fn creating_packed_data_for_one_particle_in_middle() {
        let mut store = ParticleStore::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        let particle = Particle {
            position: Vec2::new(4.5, 4.5),
            velocity: Vec2::new(1.1, 2.3),
        };
        store.add_particle(particle);
        let data = store.create_packed_data();

        #[rustfmt::skip]
        assert_eq!(
            data.indices,
            vec![
            0,  0, 0, 0,
                0, 0, 1,
                1, 1, 1,  1
            ]
        );

        assert_eq!(data.positions, vec![particle.position]);
        assert_eq!(data.velocities, vec![particle.velocity]);
    }

    #[test]
    fn creating_packed_data_for_three_particles_in_middle() {
        let mut store = ParticleStore::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        let particle = Particle {
            position: Vec2::new(3.0, 3.0),
            velocity: Vec2::new(1.1, 2.3),
        };
        store.add_particle(particle);
        store.add_particle(particle);
        store.add_particle(particle);
        let data = store.create_packed_data();
        assert_eq!(data.indices, vec![0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3]);
        assert_eq!(data.positions.len(), 3);
        assert_eq!(data.positions[1], particle.position);
        assert_eq!(data.velocities[1], particle.velocity);
    }

    #[test]
    fn creating_packed_data_for_many_particles() {
        let mut store = ParticleStore::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));

        let particle1 = Particle {
            position: Vec2::new(0.0, 1.0),
            velocity: Vec2::default(),
        };
        store.add_particle(particle1);

        let particle2 = Particle {
            position: Vec2::new(3.0, 3.0),
            velocity: Vec2::new(1.2, 3.4),
        };
        store.add_particle(particle2);

        let particle3 = Particle {
            position: Vec2::new(5.1, 4.3),
            velocity: Vec2::default(),
        };
        store.add_particle(particle3);

        let data = store.create_packed_data();
        assert_eq!(data.indices, vec![0, 0, 1, 1, 1, 1, 3, 3, 3, 3, 3]);
        assert_eq!(data.positions.len(), 3);
        assert_eq!(data.positions[2], particle3.position);
        assert_eq!(data.velocities[1], particle2.velocity);
    }

    #[test]
    fn creating_packed_data_for_a_particle_offscreen() {
        let mut store = ParticleStore::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        store.add_particle(Particle {
            position: Vec2::new(6.1, 6.1),
            velocity: Vec2::default(),
        });
        store.add_particle(Particle {
            position: Vec2::new(9.1, 9.1),
            velocity: Vec2::default(),
        });
        let data = store.create_packed_data();
        assert_eq!(data.indices, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(data.positions, vec![Vec2::new(6.1, 6.1)]);
        assert_eq!(data.velocities, vec![Vec2::default()]);
    }

    #[test]
    fn max_particles_per_frame() {
        let store = ParticleStore::new(2, Vec4::new(0.0, 0.0, 6.0, 6.0));
        assert_eq!(store.max_particles_per_frame(), 74);
    }
}
