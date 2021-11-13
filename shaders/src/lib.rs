#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

pub mod wrach_glam;
use wrach_glam::glam::{vec2, vec4, UVec3, Vec2, Vec4};

mod particle;
pub use particle::Particle;

pub const NUM_PARTICLES: usize = 1_000_000;

pub const MAP_WIDTH: u16 = 1600;
pub const MAP_HEIGHT: u16 = 800;
pub const MAP_SIZE: usize = MAP_WIDTH as usize * MAP_HEIGHT as usize;

pub type PixelMap = [u32; MAP_SIZE];

pub struct SimParams {
    _delta_t: f32,
    _rule1_distance: f32,
    _rule2_distance: f32,
    _rule3_distance: f32,
    _rule1_scale: f32,
    _rule2_scale: f32,
    _rule3_scale: f32,
}

pub struct Particles {
    particles: [Particle; NUM_PARTICLES],
}

#[spirv(compute(threads(64)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pixel_map: &mut PixelMap,
) {
    let index = id.x as usize;
    let position = particles.particles[index].position;
    let coord_f32: f32 = (position.y.floor() * MAP_WIDTH as f32) + position.x.floor();
    let coord: usize = coord_f32 as usize;
    pixel_map[coord] = index as u32;
}

#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pixel_map: &mut PixelMap,
) {
    let total = particles_src.particles.len();
    let index = id.x as usize;
    if index >= total {
        return;
    }

    let mut this_particle = particles_src.particles[index];

    let mut total_force = vec2(0.0, 0.0);

    let vacinity = 15.0;
    let x = this_particle.position.x.floor();
    let x_min = (x - vacinity) as usize;
    let x_max = (x + vacinity) as usize;
    let y = this_particle.position.y.floor();
    let y_min = (y - vacinity) as usize;
    let y_max = (y + vacinity) as usize;
    for y in y_min..y_max {
        for x in x_min..x_max {
            let coord = ((y * MAP_WIDTH as usize) + x) as usize;
            let other_particle_index = pixel_map[coord] as usize;
            if other_particle_index == index {
                continue;
            }
            if other_particle_index == 0 {
                continue;
            }
            let other_particle = particles_src.particles[other_particle_index];

            total_force += this_particle.force(other_particle);
        }
    }

    this_particle.velocity += total_force;
    this_particle.position += this_particle.velocity;

    this_particle.bounce_off_walls();

    // Write back
    particles_dst.particles[index].position = this_particle.position;
    particles_dst.particles[index].velocity = this_particle.velocity;
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
