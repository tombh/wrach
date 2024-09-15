//! Integrate

use spirv_std::glam::Vec2;

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
    pub dimensions: Vec2,
    /// Current position of the viewoport. Measured from the bottom-left corner
    pub view_anchor: Vec2,
}

/// Integrate
pub fn main(
    index: usize,
    world_config: &WorldConfig,
    positions_input: &[Vec2],
    positions_output: &mut [Vec2],
    velocities_input: &[Vec2],
    velocities_output: &mut [Vec2],
) {
    let mut particle = Particle::new(index, positions_input, velocities_input);

    particle.enforce_limits(world_config);
    particle.integrate();
    particle.write(positions_output, velocities_output);
}
