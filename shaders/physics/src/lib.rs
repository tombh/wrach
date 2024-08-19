//! Wrach physics shaders

#![no_std]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::as_conversions)]
#![allow(clippy::indexing_slicing)]

mod integrate;

use spirv_std::{
    glam::{UVec3, Vec2},
    spirv,
};

/// Integration entrypoint
#[spirv(compute(threads(1)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] positions_input: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] positions_output: &mut [Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] velocities_input: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] velocities_output: &mut [Vec2],
) {
    let index = (id.x * 8) as usize + id.y as usize;
    integrate::main(
        index,
        positions_input,
        positions_output,
        velocities_input,
        velocities_output,
    );
}
