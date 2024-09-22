//! Integrate

use spirv_std::{
    arch::IndexUnchecked,
    glam::{UVec2, Vec2},
};

use crate::{particle::Particle, PREFIX_SUM_HACK};

// TODO: I tried to make this struct shareable with the main app but get an error from `encase`
// saying that Bevy's `Vec2` doesn't implement `ShaderSized`.
//
// #[cfg(not(target_arch = "spirv"))]
// use bevy::prelude::Vec2 as ShareableVec2;
// #[cfg(not(target_arch = "spirv"))]
// use bevy::render::render_resource::ShaderType;
//
// #[cfg(target_arch = "spirv")]
// use spirv_std::glam::Vec2 as ShareableVec2;
//
// /// Config for the simulation
// #[cfg_attr(
//     not(target_arch = "spirv"),
//     derive(ShaderType, bytemuck::Zeroable, bytemuck::Pod, Copy, Clone)
// )]
// #[repr(C)]
/// Config needed by the simulation
pub struct WorldSettings {
    /// Dimensions of the view onto the simulation
    pub view_dimensions: Vec2,
    /// Current position of the viewoport. Measured from the bottom-left corner
    pub view_anchor: Vec2,
    /// The dimensions of the spatial bin grid, the unit is a cell
    pub grid_dimensions: UVec2,
    /// The size of a spatial bin cell
    pub cell_size: u32,
    /// Total number of particles simulated in this frame. This will normally be much smaller than
    /// the total number of particles that we have a record of.
    pub particles_in_frame_count: u32,
}

/// Integrate
pub fn main(
    cell_index: usize,
    settings: &WorldSettings,
    indices: &mut [u32],
    positions_input: &[Vec2],
    positions_output: &mut [Vec2],
    velocities_input: &[Vec2],
    velocities_output: &mut [Vec2],
) {
    let total_cells = settings.grid_dimensions.x * settings.grid_dimensions.y + PREFIX_SUM_HACK;
    if cell_index >= total_cells as usize {
        return;
    }

    let (particles_start_at, particles_end_at) =
        get_start_end_indices_for_particles_in_cell(indices, cell_index);

    if particles_end_at > particles_start_at {
        for particle_index in particles_start_at..=particles_end_at {
            let mut particle =
                Particle::new(particle_index as usize, positions_input, velocities_input);

            particle.integrate();
            particle.enforce_limits(settings);
            particle.write(positions_output, velocities_output);
        }
    }

    clear_indices_cell_for_next_frame(indices, cell_index);

    // This is normal, it's not because of our prefix off by one issue. It's needed because
    // of the "guard item" at the end of the prefix sum.
    if cell_index == total_cells as usize - 1 {
        clear_indices_cell_for_next_frame(indices, cell_index + 1);
    }
}

/// Clear the current value for this cell's index pointer in the indices buffer. Considering we are
/// already in the cell we save having a whole other shader and GPU pass.
fn clear_indices_cell_for_next_frame(indices: &mut [u32], cell_index: usize) {
    // SAFETY: We rely on the rest of the pipeline for correct index values.
    let cell_reference = unsafe { indices.index_unchecked_mut(cell_index) };
    *cell_reference = 0;
}

/// Based on the data structure for spatial binning, get the indices of where the first and last
/// particles of the current cell are.
fn get_start_end_indices_for_particles_in_cell(
    indices_input: &[u32],
    cell_index: usize,
) -> (u32, u32) {
    // SAFETY:
    //   Getting data with bounds checks is obviously undefined behaviour. We rely on the
    //   rest of the pipeline to ensure that indices are always within limits.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    let (particles_start_at, marker) = unsafe {
        (
            indices_input.index_unchecked(cell_index),
            indices_input.index_unchecked(cell_index + 1),
        )
    };
    let particles_count = marker - particles_start_at;
    let particles_end_at = particles_start_at + particles_count;

    (*particles_start_at, particles_end_at)
}
