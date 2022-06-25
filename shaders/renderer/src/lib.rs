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

use spirv_std::glam::{vec4, Vec2, Vec4};

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
