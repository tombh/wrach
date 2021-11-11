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

pub mod wrach_glam;
use wrach_glam::glam::{vec2, vec4, UVec3, Vec2, Vec4};

mod particle;
pub use particle::Particle;

// Number of boid particles to simulate
// Currently much more than 10000 freezes up my GPU :/
pub const NUM_PARTICLES: usize = 10000;

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
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut Particles,
) {
    let total = particles_src.particles.len();
    let index = id.x as usize;
    if index >= total {
        return;
    }

    let mut this_particle = particles_src.particles[index];

    let mut total_force = vec2(0.0, 0.0);

    let mut i: usize = 0;
    loop {
        if i >= total {
            break;
        }
        if i == index {
            i = i + 1;
            continue;
        }
        let other_particle = particles_src.particles[i];

        total_force += this_particle.force(other_particle);
        i = i + 1;
    }

    this_particle.velocity += total_force;
    this_particle.position += this_particle.velocity;

    this_particle.bounce_off_walls();

    // Write back
    *particles_dst.particles[index].position = *this_particle.position;
    *particles_dst.particles[index].velocity = *this_particle.velocity;
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
