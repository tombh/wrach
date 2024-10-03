//! Handle particles interacting with each other

use spirv_std::{arch::IndexUnchecked, glam::Vec2};
use wrach_cpu_gpu_shared::WorldSettings;

use crate::{
    cell::MAX_PARTICLES_IN_CELL,
    indices::{Indices, Range, Storage},
    particle::Particle,
    WORKGROUP_MEMORY_SIZE,
};

/// The number of cells that are considered for physics interactions.
const REQUIRED_CELLS: usize = 2;

/// The minimum distance allowed between particles
pub const MIN_DISTANCE: f32 = 1.0;

/// All the particles in a cell.
pub struct Particles {
    /// All the global and local indices that point to where we find the particles for a cell.
    pub indices: Indices,
    /// A local array of particles to check for interactions. Because multiple particles will be
    /// checked multiple times, hopefully we save some global memory read latency by only reading them
    /// once then doing physics on them locally.
    //
    // PERFORMANCE:
    //   It might be better to put these in workgroup memory. Because when local memory overflows,
    //   then global memory is used.
    pub data: [Particle; MAX_PARTICLES_IN_CELL * REQUIRED_CELLS],
    /// Number of particles in the centre cell.
    pub centre_count: usize,
}

impl Particles {
    /// Instantiate
    pub fn new(
        indices_main: &[u32],
        indices_aux: &[u32],
        cell_index: usize,
        grid_width: u32,
        workgroup_offset: usize,

        positions_in: &[Vec2],
        velocities_in: &[Vec2],
        positions_aux: &[Vec2; WORKGROUP_MEMORY_SIZE],
        velocities_aux: &[Vec2; WORKGROUP_MEMORY_SIZE],
    ) -> Self {
        let mut particles = Self {
            indices: Indices::new(
                indices_main,
                indices_aux,
                cell_index,
                grid_width,
                workgroup_offset,
            ),
            data: [Particle::default(); MAX_PARTICLES_IN_CELL * REQUIRED_CELLS],
            centre_count: 0,
        };

        particles.centre_count = particles.load_particles_from_cell_centre(
            particles.indices.centre,
            positions_in,
            velocities_in,
        );
        // particles.load_particles_from_cell(
        //     particles.indices.bottom_left,
        //     positions_aux,
        //     velocities_aux,
        // );
        // particles.load_particles_from_cell(
        //     particles.indices.bottom_right,
        //     positions_aux,
        //     velocities_aux,
        // );
        // particles.load_particles_from_cell(
        //     particles.indices.top_left,
        //     positions_aux,
        //     velocities_aux,
        // );
        particles.load_particles_from_cell(
            particles.indices.top_right,
            positions_aux,
            velocities_aux,
        );

        particles
    }

    /// asdf as
    fn load_particles_from_cell_centre(
        &mut self,
        storage: Storage,
        positions: &[Vec2],
        velocities: &[Vec2],
    ) -> usize {
        let mut particles_count = storage.global.end_by - storage.global.from;
        if particles_count > MAX_PARTICLES_IN_CELL {
            particles_count = MAX_PARTICLES_IN_CELL;
        }

        let particles_end_by = storage.global.from + particles_count;

        let mut local_index = storage.local.from;
        for global_index in storage.global.from..particles_end_by {
            self.set(
                local_index,
                Particle::new(global_index, positions, velocities),
            );
            local_index += 1;
        }

        particles_count
    }

    /// Add particles from cell.
    fn load_particles_from_cell(
        &mut self,
        storage: Storage,
        positions: &[Vec2; WORKGROUP_MEMORY_SIZE],
        velocities: &[Vec2; WORKGROUP_MEMORY_SIZE],
    ) -> usize {
        let mut particles_count = storage.global.end_by - storage.global.from;
        if particles_count > MAX_PARTICLES_IN_CELL {
            particles_count = MAX_PARTICLES_IN_CELL;
        }

        let particles_end_by = storage.global.from + particles_count;

        let mut local_index = storage.local.from;
        for global_index in storage.global.from..particles_end_by {
            self.set(
                local_index,
                Particle::new_aux(global_index, positions, velocities),
            );
            local_index += 1;
        }

        particles_count
    }

    /// Calculate the distance between 2 particles.
    fn calculate_distance_between_particle_pair(
        &mut self,
        index_left: usize,
        index_right: usize,
    ) -> f32 {
        let mut distance = self
            .particle(index_left)
            .position
            .distance(self.particle(index_right).position);

        if distance == 0.0 {
            distance = 0.0001;
        }

        distance
    }

    /// Iterate through unique pairs of particles in the centre cell and do physics on them.
    pub fn pairs_in_cell(&mut self) {
        for i_left in 0..self.centre_count {
            for i_right in (i_left + 1)..self.centre_count {
                let distance = self.calculate_distance_between_particle_pair(i_left, i_right);
                if distance > MIN_DISTANCE {
                    continue;
                }

                self.push_close_particles_apart(distance, i_left, i_right);
            }
        }
    }

    /// Iterate through all the particles that immediately surround the cell. We only write back to
    /// the main cell particles because the auxiliary particles are in a read-only buffer.
    pub fn auxiliares_around_cell(&mut self) {
        // self.single_auxiliary_cell(self.indices.bottom_left.local);
        // self.single_auxiliary_cell(self.indices.bottom_right.local);
        // self.single_auxiliary_cell(self.indices.top_left.local);
        // self.single_auxiliary_cell(self.indices.top_right.local);
    }

    /// With all the particles in the main cell, iterate over the them and the particles in an
    /// auxiliary cell.
    pub fn single_auxiliary_cell(&mut self, indices: Range) {
        if indices.end_by - indices.from > MAX_PARTICLES_IN_CELL {
            return;
        }
        for i_left in 0..self.centre_count {
            for i_right in indices.from..indices.end_by {
                let distance = self.calculate_distance_between_particle_pair(i_left, i_right);
                if distance > MIN_DISTANCE {
                    continue;
                }

                self.push_close_particles_apart(distance, i_left, i_right);
            }
        }
    }

    /// If 2 particles are closer than their size allows then just forcefully move them apart to a
    /// safe distance.
    fn push_close_particles_apart(&mut self, distance: f32, i_left: usize, i_right: usize) {
        let force = 0.5 * (MIN_DISTANCE - distance) / distance;
        let mut distance_vec: Vec2 =
            self.particle(i_right).position - self.particle(i_left).position;
        distance_vec *= force;

        self.particle(i_left).position -= distance_vec;
        self.particle(i_right).position += distance_vec;
    }

    /// Integrate the final values and write them back to VRAM.
    pub fn finish(
        &mut self,
        settings: &WorldSettings,
        positions: &mut [Vec2],
        velocities: &mut [Vec2],
    ) {
        for i in 0..self.centre_count {
            self.particle(i).integrate();
            self.particle(i).enforce_limits(settings);
            self.particle(i).write(positions, velocities);
        }
    }

    /// Set particle data locally.
    fn set(&mut self, index: usize, value: Particle) {
        // SAFETY: We're relying on surrounding code to make sure this is never out of bounds.
        let particle_ref = unsafe { self.data.index_unchecked_mut(index) };
        *particle_ref = value;
    }

    /// Get particle data from local array.
    fn particle(&mut self, index: usize) -> &mut Particle {
        // SAFETY: We're relying on surrounding code to make sure this is never out of bounds.
        unsafe { self.data.index_unchecked_mut(index) }
    }
}

// #[allow(clippy::unreadable_literal)]
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     fn setup(cell_index: usize) -> Particles {
//         #[rustfmt::skip]
//         let indices_main = [
//             0,  2, 2,
//                 2, 3
//         ];
//         #[rustfmt::skip]
//         let indices_aux = [
//             0,  0, 0, 0,
//                 0, 0, 0,
//                 0, 0, 1
//         ];
//
//         let positions_main = &[
//             Vec2::new(1.0, 1.0),
//             Vec2::new(1.1, 1.1),
//             Vec2::new(10.0, 10.0),
//         ];
//         let velocities_main = &[
//             Vec2::new(0.1, 0.2),
//             Vec2::new(0.3, 0.4),
//             Vec2::new(0.5, 0.6),
//         ];
//         let positions_aux = &[Vec2::new(10.1, 10.1)];
//         let velocities_aux = &[Vec2::new(0.1, 0.2)];
//
//         Particles::new(
//             &indices_main,
//             &indices_aux,
//             cell_index,
//             2,
//             0,
//             positions_main,
//             velocities_main,
//             positions_aux,
//             velocities_aux,
//         )
//     }
//
//     #[test]
//     fn pushes_particles_apart_in_main_cell() {
//         let mut particles = setup(0);
//
//         particles.pairs_in_cell();
//
//         assert_eq!(
//             particles.data[0].position,
//             Vec2::new(0.69644666, 0.69644666)
//         );
//         assert_eq!(particles.data[1].position, Vec2::new(1.4035534, 1.4035534));
//
//         let new_distance = particles.data[0]
//             .position
//             .distance(particles.data[1].position);
//         assert!(new_distance < 1.001);
//         assert!(new_distance > 0.999);
//     }
//
//     #[test]
//     #[allow(clippy::excessive_precision)]
//     fn pushes_particles_apart_between_main_and_aux_cell() {
//         let mut particles = setup(3);
//
//         particles.auxiliares_around_cell();
//
//         let cell_particle = particles.data[0];
//         let aux_particle = particles.data[16];
//
//         assert_eq!(cell_particle.position, Vec2::new(9.69644666, 9.69644666));
//         assert_eq!(aux_particle.position, Vec2::new(10.4035539, 10.4035539));
//
//         let new_distance = cell_particle.position.distance(aux_particle.position);
//         assert!(new_distance < 1.001);
//         assert!(new_distance > 0.999);
//     }
// }
