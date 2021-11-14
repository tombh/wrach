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

#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

pub mod compute;
mod particle;
pub mod wrach_glam;

use wrach_glam::glam::{vec4, UVec3, Vec2, Vec4};

#[spirv(compute(threads(64)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles: &mut compute::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)]
    _particles_dst: &mut compute::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pixel_map: &mut compute::PixelMap,
) {
    let index = id.x as usize;
    let position = particles.particles[index].position;
    let coord_f32: f32 = (position.y.floor() * compute::MAP_WIDTH as f32) + position.x.floor();
    let coord: usize = coord_f32 as usize;
    pixel_map[coord] = index as u32;
}

#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &compute::SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    particles_src: &mut compute::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)]
    particles_dst: &mut compute::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pixel_map: &mut compute::PixelMap,
) {
    compute::entry(id, params, particles_src, particles_dst, pixel_map);
}

// Called for every index of a vertex, there are 6 in a square, because a square
// is made up from 2 triangles
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vert_id: i32,
    particle_position: Vec2,
    // interesting to consider how other properties could "shape" pixels
    _particle_velocity: Vec2,
    vertex: Vec2,
    #[spirv(position)] screen_position: &mut Vec4,
) {
    *screen_position = vec4(
        particle_position.x + vertex.x,
        particle_position.y + vertex.y,
        0.0,
        1.0,
    );
}

// Basically just the colour
#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4) {
    *output = vec4(1.0, 0.0, 0.0, 1.0);
}
