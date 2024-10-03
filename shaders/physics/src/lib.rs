//! Wrach physics shaders

#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::as_conversions)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::needless_range_loop)]

use cell::{Cell, MAX_PARTICLES_IN_CELL};
use spirv_std::{
    arch::workgroup_memory_barrier_with_group_sync,
    glam::{UVec3, Vec2},
    spirv,
};
use wrach_cpu_gpu_shared::{WorldSettings, PREFIX_SUM_OFFSET_HACK};

mod cell;
mod indices;
mod particle;
mod particles;

const THREADS_PER_WORKGROUP: u32 = 64;
pub const WORKGROUP_MEMORY_SIZE: usize = THREADS_PER_WORKGROUP as usize * MAX_PARTICLES_IN_CELL * 4;

/// Physics entrypoint
#[spirv(compute(threads(64)))]
pub fn main(
    // #[spirv(global_invocation_id)] global_invocation: UVec3,
    #[spirv(workgroup_id)] workgroup_invocation: UVec3,
    #[spirv(local_invocation_id)] local_invocation: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &WorldSettings,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] indices_main: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] positions_in: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] positions_out: &mut [Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] velocities_in: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] velocities_out: &mut [Vec2],

    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] indices_aux: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] positions_aux: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)] velocities_aux: &[Vec2],

    #[spirv(workgroup)] positions_workgroup: &mut [Vec2; WORKGROUP_MEMORY_SIZE],
    #[spirv(workgroup)] velocities_workgroup: &mut [Vec2; WORKGROUP_MEMORY_SIZE],
) {
    // let current_cell = (global_invocation.x + PREFIX_SUM_OFFSET_HACK) as usize;
    let first_cell_in_workgroup = workgroup_invocation.x * THREADS_PER_WORKGROUP;
    let last_cell_in_workgroup = first_cell_in_workgroup + THREADS_PER_WORKGROUP - 1;

    let current_cell =
        (first_cell_in_workgroup + local_invocation.x + PREFIX_SUM_OFFSET_HACK) as usize;

    let mut first_particle_index: usize = 0;
    if local_invocation.x == 1 {
        let grid_width = settings.grid_dimensions.x;
        let aux_steps = [0, 1, grid_width, grid_width + 1];
        let mut workgroup_memory_index = 0;
        for cell_index in first_cell_in_workgroup..=last_cell_in_workgroup {
            for s in 0..4 {
                let step = aux_steps[s];
                let aux_index = (cell_index + step) as usize;
                let (particles_start_at, marker) =
                    (indices_aux[aux_index], indices_aux[aux_index + 1]);
                if first_particle_index == 0 {
                    first_particle_index = particles_start_at as usize;
                }
                let particles_count = marker - particles_start_at;
                let particles_end_by = particles_start_at + particles_count;
                for particle_index in particles_start_at..particles_end_by {
                    positions_workgroup[workgroup_memory_index] =
                        positions_aux[particle_index as usize];
                    velocities_workgroup[workgroup_memory_index] =
                        velocities_aux[particle_index as usize];

                    workgroup_memory_index += 1;
                }
            }
        }
    }

    // Safety: I think `unsafe` is required because this can be difficult to manage correctly.
    unsafe {
        workgroup_memory_barrier_with_group_sync();
    };

    let mut world = Cell {
        current: current_cell,
        is_last: false,
        first_particle_index,
        settings,

        indices_main,
        positions_in,
        positions_out,
        velocities_in,
        velocities_out,

        indices_aux,
        positions_workgroup,
        velocities_workgroup,
    };

    world.physics_for_cell();
}
