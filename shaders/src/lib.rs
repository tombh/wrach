#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

pub mod compute;
pub mod neighbours;
pub mod particle;
pub mod world;
pub mod wrach_glam;

use wrach_glam::glam::{vec4, UVec3, Vec2, Vec4};

#[rustfmt::skip]
#[spirv(compute(threads(64)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] grid: &mut neighbours::GridBasic,
) {
    let grid_id = id.x;
    if grid_id + 1 > neighbours::GRID_COUNT as u32 {
        return;
    }
    neighbours::NeighbouringParticles::populate_grid(grid_id, particles_src, grid);
}

#[rustfmt::skip]
#[spirv(compute(threads(64)))]
pub fn predict_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] grid: &neighbours::GridBasic,
) {
    compute::entry(id, params, particles_src, particles_dst, grid, 0);
}

#[rustfmt::skip]
#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] grid: &neighbours::GridBasic,
) {
    compute::entry(id, params, particles_src, particles_dst, grid, 1);
}

#[rustfmt::skip]
#[spirv(compute(threads(64)))]
pub fn post_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] grid: &neighbours::GridBasic,
) {
    compute::entry(id, params, particles_src, particles_dst, grid, 2);
}

// Called for every index of a vertex, there are 6 in a square, because a square
// is made up from 2 triangles
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vert_id: i32,
    #[spirv(position)] screen_position: &mut Vec4,
    particle_color: Vec4,
    particle_position: Vec2,
    _particle_velocity: Vec2,
    _particle_gradient: Vec2,
    vertex: Vec2,
    output: &mut Vec4,
) {
    *screen_position = vec4(
        particle_position.x + vertex.x,
        particle_position.y + vertex.y,
        0.0,
        1.0,
    );
    *output = vec4(
        particle_color.x,
        particle_color.y,
        particle_color.z,
        particle_color.w,
    );
}

// Basically just the colour
#[spirv(fragment)]
pub fn main_fs(input: Vec4, output: &mut Vec4) {
    *output = vec4(input.x, input.y, input.z, input.w);
}
