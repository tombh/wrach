//! Wrach physics shaders

#![no_std]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::as_conversions)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::needless_range_loop)]

use cell::World;
use spirv_std::{
    glam::{UVec3, Vec2},
    spirv,
};
use wrach_cpu_gpu_shared::WorldSettings;

mod cell;
mod particle;
mod particles;

/// NB: We add one to cell indexes because our current prefix sum implementation shifts all its items
/// one to the right.
pub const PREFIX_SUM_HACK: u32 = 1;

/// Physics entrypoint
#[spirv(compute(threads(32)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &WorldSettings,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] indices: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] positions_input: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] positions_output: &mut [Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] velocities_input: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] velocities_output: &mut [Vec2],
) {
    let current_cell = (id.x + PREFIX_SUM_HACK) as usize;

    let mut world = World {
        current_cell,
        settings,
        indices,
        positions_input,
        positions_output,
        velocities_input,
        velocities_output,
    };

    world.physics_for_cell();
}
