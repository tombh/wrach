//! A cell is the unit of work in our GPU compute workload. A single work item loads all the
//! particles in a spatial bin cell (and its surroundings) and does physics on this particles.

use spirv_std::{arch::IndexUnchecked as _, glam::Vec2};

use wrach_cpu_gpu_shared::{self as shared, WorldSettings};

use crate::{particle::Particle, particles::Particles, PREFIX_SUM_HACK};

/// The amount of extra space for over-packed cells. If the `MIN_DISTANCE` is right then this
/// should not generally be needed. I think it's most useful for the very beginning of a simulation
/// when the particles have been randomly placed and there's a chance that some cells are
/// over-packed because their particles haven't been pushed apart yet.
///
/// I'm not quite sure what the implications are for a cell that has more particles than we've
/// reserved space for. I think all that happens is that some particles get missed, and so get
/// picked up as normal in another frame.
///
/// The only performance concerns for this should be memory size. There is an extra loop to at
/// least do basic velocity calculations and copy the particle to the destination buffer.
const CELL_LEEWAY: f32 = 1.0;

/// The maximum number of particles we can handle in a cell.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "I don't think these are issues"
)]
pub const MAX_PARTICLES_IN_CELL: usize =
    (shared::SPATIAL_BIN_CELL_SIZE.pow(2) as f32 * CELL_LEEWAY) as usize;

/// All the data needed to simulate the particle world.
pub struct World<'world> {
    /// The index of the current spatial bin cell.
    pub current_cell: usize,
    /// Config, like viewport position etc.
    pub settings: &'world WorldSettings,
    /// An array of spatial bin cells and how many particles each contains.
    pub indices: &'world mut [u32],
    /// Particle positions for reading.
    pub positions_input: &'world [Vec2],
    /// Particle positions for writing.
    pub positions_output: &'world mut [Vec2],
    /// Velocity positions for reading.
    pub velocities_input: &'world [Vec2],
    /// Velocity positions for writing.
    pub velocities_output: &'world mut [Vec2],
}

impl World<'_> {
    /// Iterate over all the particles in a cell and do physics on them.
    pub fn physics_for_cell(&mut self) {
        let total_cells =
            self.settings.grid_dimensions.x * self.settings.grid_dimensions.y + PREFIX_SUM_HACK;
        let last_cell = total_cells as usize - 1;
        if self.current_cell > last_cell {
            return;
        }

        let (particles_start_at, all_particles_count) =
            self.get_start_end_indices_for_particles_in_cell();

        let mut particles = Particles::new(
            particles_start_at,
            all_particles_count,
            self.positions_input,
            self.velocities_input,
        );
        particles.pairs();
        particles.finish(self.settings, self.positions_output, self.velocities_output);

        self.handle_overflown_particles(particles_start_at, particles.count, all_particles_count);

        self.clear_cells_for_next_frame(last_cell);
    }

    /// Handle particles that overflew the [`MAX_PARTICLES_IN_CELL`] limit. They should at least be
    /// integrated and copied back to VRAM. They'll likely be picked up in the next frame.
    fn handle_overflown_particles(
        &mut self,
        particles_start_at: usize,
        particles_count: usize,
        all_particles_count: usize,
    ) {
        let particles_end_at = particles_start_at + particles_count;
        let all_particles_end_at = particles_start_at + all_particles_count;

        for particle_index in particles_end_at..all_particles_end_at {
            let mut particle =
                Particle::new(particle_index, self.positions_input, self.velocities_input);
            particle.integrate();
            particle.enforce_limits(self.settings);
            particle.write(self.positions_output, self.velocities_output);
        }
    }

    /// Based on the data structure for spatial binning, get the indices of where the first and last
    /// particles of the current cell are.
    fn get_start_end_indices_for_particles_in_cell(&self) -> (usize, usize) {
        // SAFETY:
        //   Getting data with bounds checks is obviously undefined behaviour. We rely on the
        //   rest of the pipeline to ensure that indices are always within limits.
        let (particles_start_at, marker) = unsafe {
            (
                self.indices.index_unchecked(self.current_cell),
                self.indices.index_unchecked(self.current_cell + 1),
            )
        };
        let particles_count = marker - particles_start_at;

        (*particles_start_at as usize, particles_count as usize)
    }

    /// The spatial bin cells recieve atomically added particle counts, so they need to all be set
    /// to zero at the beginning of each pass.
    fn clear_cells_for_next_frame(&mut self, last_cell: usize) {
        self.clear_indices_cell_for_next_frame(self.current_cell);
        // This is normal, it's not because of our prefix off-by-one issue. It's needed because
        // of the "guard item" at the end of the prefix sum.
        if self.current_cell == last_cell {
            self.clear_indices_cell_for_next_frame(self.current_cell + 1);
        }
    }

    /// Clear the current value for this cell's index pointer in the indices buffer. Considering we are
    /// already in the cell we save having a whole other shader and GPU pass.
    fn clear_indices_cell_for_next_frame(&mut self, cell_index: usize) {
        // SAFETY: We rely on the rest of the pipeline for correct index values.
        let cell_reference = unsafe { self.indices.index_unchecked_mut(cell_index) };
        *cell_reference = 0;
    }
}
