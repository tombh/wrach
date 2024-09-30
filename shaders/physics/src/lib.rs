//! Wrach physics shaders

#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::as_conversions)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::needless_range_loop)]

use cell::Cell;
use spirv_std::{
    glam::{UVec3, Vec2},
    spirv,
};
use wrach_cpu_gpu_shared::{WorldSettings, PREFIX_SUM_OFFSET_HACK};

mod cell;
mod indices;
mod particle;
mod particles;

/// Physics entrypoint
#[spirv(compute(threads(64)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &WorldSettings,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] indices_main: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] positions_in: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] positions_out: &mut [Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] velocities_in: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] velocities_out: &mut [Vec2],

    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] indices_aux: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] positions_aux: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)] velocities_aux: &[Vec2],
) {
    let current_cell = (id.x + PREFIX_SUM_OFFSET_HACK) as usize;

    let mut world = Cell {
        current: current_cell,
        is_last: false,
        settings,

        indices_main,
        positions_in,
        positions_out,
        velocities_in,
        velocities_out,

        indices_aux,
        positions_aux,
        velocities_aux,
    };

    world.physics_for_cell();
}
