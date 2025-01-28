//! A cell is the unit of work in our GPU compute workload. A single work item loads all the
//! particles in a spatial bin cell (and its surroundings) and does physics on this particles.

use spirv_std::{arch::IndexUnchecked as _, glam::Vec2, num_traits::Euclid};

use wrach_cpu_gpu_shared::{self as shared, WorldSettings};

use crate::{particle::Particle, particles::Particles, PREFIX_SUM_OFFSET_HACK};

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
pub struct Cell<'world> {
    /// The index of the current spatial bin cell.
    pub current: usize,
    /// Is this the final spatial bin cell?
    pub is_last: bool,

    pub first_particle_index: usize,

    /// Config, like viewport position etc.
    pub settings: &'world WorldSettings,

    /// An array of spatial bin cells and how many particles each contains.
    pub indices_main: &'world mut [u32],
    /// Particle positions for reading.
    pub positions_in: &'world [Vec2],
    /// Particle positions for writing.
    pub positions_out: &'world mut [Vec2],
    /// Velocity positions for reading.
    pub velocities_in: &'world [Vec2],
    /// Velocity positions for writing.
    pub velocities_out: &'world mut [Vec2],

    /// Indices of aux particles that surround main cells.
    pub indices_aux: &'world mut [u32],
    /// Aux cell particle positions for reading.
    pub positions_workgroup: &'world [Vec2; 1024],
    /// Aux cell particle positions for writing.
    pub velocities_workgroup: &'world [Vec2; 1024],
}

impl Cell<'_> {
    /// Iterate over all the particles in a cell and do physics on them.
    pub fn physics_for_cell(&mut self) {
        let total_cells = self.settings.grid_dimensions.x * self.settings.grid_dimensions.y
            + PREFIX_SUM_OFFSET_HACK;
        let last_cell_index = total_cells as usize - 1;
        if self.current > last_cell_index {
            return;
        }
        self.is_last = self.current == last_cell_index;

        let mut particles = Particles::new(
            self.indices_main,
            self.indices_aux,
            self.current,
            self.settings.grid_dimensions.x,
            self.first_particle_index,
            self.positions_in,
            self.velocities_in,
            self.positions_workgroup,
            self.velocities_workgroup,
        );
        particles.pairs_in_cell();
        particles.auxiliares_around_cell();
        particles.finish(self.settings, self.positions_out, self.velocities_out);

        self.handle_overflown_particles(
            particles.indices.centre.global.from,
            particles.indices.centre.global.end_by,
        );
        self.clear_cells_for_next_frame();
    }

    /// Handle particles that overflew the [`MAX_PARTICLES_IN_CELL`] limit. They should at least be
    /// integrated and copied back to VRAM. They'll likely be picked up in the next frame.
    fn handle_overflown_particles(&mut self, particles_start_at: usize, particles_end_by: usize) {
        let overflow_starts_at = particles_start_at + MAX_PARTICLES_IN_CELL;
        if particles_end_by <= overflow_starts_at {
            return;
        }
        for particle_index in overflow_starts_at..particles_end_by {
            let mut particle = Particle::new(particle_index, self.positions_in, self.velocities_in);
            particle.integrate();
            particle.enforce_limits(self.settings);
            particle.write(self.positions_out, self.velocities_out);
        }
    }

    // /// Based on the data structure for spatial binning, get the indices of where the first and last
    // /// particles of the current cell are.
    // fn get_start_end_indices_for_particles_in_cell(&self) -> (usize, usize) {
    //     // SAFETY:
    //     //   Getting data with bounds checks is obviously undefined behaviour. We rely on the
    //     //   rest of the pipeline to ensure that indices are always within limits.
    //     let (particles_start_at, marker) = unsafe {
    //         (
    //             self.indices.index_unchecked(self.current),
    //             self.indices.index_unchecked(self.current + 1),
    //         )
    //     };
    //     let particles_count = marker - particles_start_at;
    //
    //     (*particles_start_at as usize, particles_count as usize)
    // }

    /// The spatial bin cells receive atomically added particle counts, so they need to all be set
    /// to zero at the beginning of each pass.
    fn clear_cells_for_next_frame(&mut self) {
        self.clear_indices_cells_for_next_frame(self.current);
        // This is normal, it's not because of our prefix off-by-one issue. It's needed because
        // of the "guard item" at the end of the prefix sum.
        if self.is_last {
            self.clear_indices_cells_for_next_frame(self.current + 1);
        }
    }

    /// Clear the current value for this cell's index pointer in the indices buffer. Considering we are
    /// already in the cell we save having a whole other shader and GPU pass.
    fn clear_indices_cells_for_next_frame(&mut self, cell_index: usize) {
        // SAFETY: We rely on the rest of the pipeline for correct index values.
        unsafe {
            let main_cell_reference = self.indices_main.index_unchecked_mut(cell_index);
            *main_cell_reference = 0;

            let aux_cell_reference = self.indices_aux.index_unchecked_mut(cell_index);
            *aux_cell_reference = 0;

            if self.is_last {
                return;
            }

            if self.is_cell_at_right_edge(cell_index) {
                let aux_right_cell_reference = self.indices_aux.index_unchecked_mut(cell_index + 1);
                *aux_right_cell_reference = 0;
            };

            if self.is_cell_at_top_edge(cell_index) {
                let width = self.settings.grid_dimensions.x as usize;
                let aux_right_cell_reference =
                    self.indices_aux.index_unchecked_mut(cell_index + width);
                *aux_right_cell_reference = 0;
            };
        };
    }

    /// Is the cell at the right edge of the grid?
    fn is_cell_at_right_edge(&self, cell_index: usize) -> bool {
        let width = self.settings.grid_dimensions.x as usize;
        let (_, remainder) = cell_index.div_rem_euclid(&width);
        remainder == width - 1
    }

    /// Is the cell at the top edge of the grid?
    const fn is_cell_at_top_edge(&self, cell_index: usize) -> bool {
        let height = self.settings.grid_dimensions.y as usize;
        let rows = cell_index.div_euclid(height);
        rows == height - 1
    }
}

// #[cfg(test)]
// mod tests {
//
//     use bevy::math::{UVec2, Vec2};
//
//     use super::*;
//
//     #[test]
//     fn border_checks() {
//         let cell = Cell {
//             current: 0,
//             is_last: false,
//             first_particle_index: 0,
//             settings: &WorldSettings {
//                 view_dimensions: Vec2::ZERO,
//                 view_anchor: Vec2::ZERO,
//                 grid_dimensions: UVec2::new(10, 10),
//                 cell_size: 2,
//                 particles_in_frame_count: 0,
//             },
//
//             indices_main: &mut [],
//             positions_in: &[],
//             velocities_in: &[],
//             positions_out: &mut [],
//             velocities_out: &mut [],
//
//             indices_aux: &mut [],
//             positions_workgroup: &mut [],
//             velocities_workgroup: &[],
//         };
//
//         assert!(!cell.is_cell_at_right_edge(0));
//         assert!(!cell.is_cell_at_right_edge(1));
//         assert!(cell.is_cell_at_right_edge(9));
//         assert!(cell.is_cell_at_right_edge(59));
//         assert!(cell.is_cell_at_right_edge(99));
//         assert!(!cell.is_cell_at_right_edge(100));
//
//         assert!(!cell.is_cell_at_top_edge(0));
//         assert!(!cell.is_cell_at_top_edge(55));
//         assert!(cell.is_cell_at_top_edge(90));
//         assert!(cell.is_cell_at_top_edge(99));
//         assert!(!cell.is_cell_at_top_edge(100));
//     }
// }
