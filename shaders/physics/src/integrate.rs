//! Integrate

use spirv_std::glam::Vec2;

/// Integrate
pub fn main(
    index: usize,
    positions_input: &[Vec2],
    positions_output: &mut [Vec2],
    velocities_input: &[Vec2],
    velocities_output: &mut [Vec2],
) {
    velocities_output[index] = velocities_input[index];

    if positions_input[index].x >= 1.0 || positions_input[index].x <= -1.0 {
        velocities_output[index].x = velocities_input[index].x * -1.0;
    }

    if positions_input[index].y >= 1.0 || positions_input[index].y <= -1.0 {
        velocities_output[index].y = velocities_input[index].y * -1.0;
    }

    let max = 0.01;
    velocities_output[index].x = velocities_output[index].x.clamp(-max, max);
    velocities_output[index].y = velocities_output[index].y.clamp(-max, max);

    positions_output[index] = positions_input[index] + velocities_output[index];
}
