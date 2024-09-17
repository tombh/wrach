//! Integrate

use spirv_std::{arch::IndexUnchecked, glam::Vec2};

use crate::particle::Particle;

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
pub struct WorldConfig {
    /// Dimensions of the view onto the simulation
    pub view_dimensions: Vec2,
    /// Current position of the viewport. Measured from the bottom-left corner
    pub view_anchor: Vec2,
}

/// Integrate
pub fn main(
    cell_index: usize,
    world_config: &WorldConfig,
    indices_input: &[u32],
    indices_output: &mut [u32],
    positions_input: &[Vec2],
    positions_output: &mut [Vec2],
    velocities_input: &[Vec2],
    velocities_output: &mut [Vec2],
) {
    let (particles_start_at, particles_end_at) =
        get_start_end_indices_for_particles(indices_input, cell_index);

    for particle_index in particles_start_at..=particles_end_at {
        let mut particle =
            Particle::new(particle_index as usize, positions_input, velocities_input);

        particle.integrate();
        particle.enforce_limits(world_config);
        particle.write(positions_output, velocities_output);
    }

    // TODO: hack to prevent the argument being compiled away
    indices_output[cell_index] = 0;
}

/// Based on the data structure for spatial binning, get the indices of where the first and last
/// particles of the current cell are.
fn get_start_end_indices_for_particles(indices_input: &[u32], cell_index: usize) -> (u32, u32) {
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
