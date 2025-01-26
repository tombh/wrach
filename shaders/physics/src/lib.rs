//! Wrach physics shaders

#![expect(
    stable_features,
    reason = "Remove `feature(lint_reasons)` once `rust-gpu` supports Rust 1.81"
)]
#![feature(lint_reasons)]
#![no_std]
#![expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::explicit_counter_loop,
    clippy::multiple_unsafe_ops_per_block,
    reason = "`rust-gpu` is a subset of Rust and has some unique requirements"
)]

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
#[allow(
    clippy::allow_attributes,
    reason = "For some reason `expect` doesn't detect the veracity of the 'inline' lint"
)]
#[allow(
    clippy::missing_inline_in_public_items,
    reason = "SPIR-V requires an entrypoint"
)]
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
