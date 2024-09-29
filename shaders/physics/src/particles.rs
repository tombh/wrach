//! Handle particles interacting with each other

use spirv_std::{arch::IndexUnchecked, glam::Vec2};
use wrach_cpu_gpu_shared::WorldSettings;

use crate::{cell::MAX_PARTICLES_IN_CELL, particle::Particle};

/// A local array of particles to check for interactions. Because multiple particles will be
/// checked multiple times, hopefully we save some global memory read latency by only reading them
/// once then doing physics on them locally.
//
// PERFORMANCE:
//   It might be better to put these in workgroup memory. Because when local memory overflows,
//   then global memory is used.
type LocalParticles = [Particle; MAX_PARTICLES_IN_CELL];

/// The minimum distance allowed between particles
pub const MIN_DISTANCE: f32 = 1.0;

/// All the particles in a cell.
pub struct Particles {
    /// Particle data
    pub data: LocalParticles,
    /// The number of particles that we're calculating. This number might be less than the number
    /// of particles in the cell, if the cell contains more than [`MAX_PARTICLES_IN_CELL`].
    pub count: usize,
}

impl Particles {
    /// Instantiate
    pub fn new(
        particles_start_at: usize,
        all_particles_count: usize,
        positions: &[Vec2],
        velocities: &[Vec2],
    ) -> Self {
        let mut particles_count = all_particles_count;
        if particles_count > MAX_PARTICLES_IN_CELL {
            particles_count = MAX_PARTICLES_IN_CELL;
        }

        let mut particles = Self {
            data: [Particle::default(); MAX_PARTICLES_IN_CELL],
            count: particles_count,
        };

        let particles_end_at = particles_start_at + particles_count;

        let mut local_index = 0;
        for global_index in particles_start_at..particles_end_at {
            particles.set(
                local_index,
                Particle::new(global_index, positions, velocities),
            );
            local_index += 1;
        }

        particles
    }

    /// Iterate through unique pairs of particles and do physics on them.
    pub fn pairs(&mut self) {
        for i_left in 0..self.count {
            for i_right in (i_left + 1)..self.count {
                let mut distance = self
                    .particle(i_left)
                    .position
                    .distance(self.particle(i_right).position);

                if distance > MIN_DISTANCE {
                    continue;
                }

                if distance == 0.0 {
                    distance = 0.0001;
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
        for i in 0..self.count {
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

#[allow(clippy::unreadable_literal)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pushes_particles_apart() {
        let positions = &[Vec2::new(1.0, 1.0), Vec2::new(1.1, 1.1)];
        let velocities = &[Vec2::new(0.1, 0.2), Vec2::new(0.3, 0.4)];
        let mut particles = Particles::new(0, 2, positions, velocities);
        particles.pairs();

        assert_eq!(
            particles.data[0].position,
            Vec2::new(0.69644666, 0.69644666)
        );
        assert_eq!(particles.data[1].position, Vec2::new(1.4035534, 1.4035534));

        let new_distance = particles.data[0]
            .position
            .distance(particles.data[1].position);
        assert!(new_distance < 1.001);
        assert!(new_distance > 0.999);
    }
}
