//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

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

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use glam::{vec2, vec4, UVec3, Vec2, Vec4};
pub use spirv_std::glam;

mod compute;
mod vertex;

struct Particle {
    pos: Vec2,
    vel: Vec2,
}

pub struct SimParams {
    delta_t: f32,
    rule1_distance: f32,
    rule2_distance: f32,
    rule3_distance: f32,
    rule1_scale: f32,
    rule2_scale: f32,
    rule3_scale: f32,
}

// [[stride(16)]]
pub struct Particles {
    particles: [Particle; 1500],
}

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
// LocalSize/numthreads of (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &SimParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut Particles,
) {
    let total = particles_src.particles.len();
    let index = id.x as usize;
    if index >= total {
        return;
    }

    let mut v_pos = particles_src.particles[index].pos;
    let mut v_vel = particles_src.particles[index].vel;

    let mut c_mass = vec2(0.0, 0.0);
    let mut c_vel = vec2(0.0, 0.0);
    let mut col_vel = vec2(0.0, 0.0);
    let mut c_mass_count: i32 = 0;
    let mut c_vel_count: i32 = 0;

    let mut i: usize = 0;
    loop {
        if i >= total {
            break;
        }
        if i == index {
            i = i + 1;
            continue;
        }

        let pos = particles_src.particles[i].pos;
        let vel = particles_src.particles[i].vel;

        if pos.distance(v_pos) < params.rule1_distance {
            c_mass = c_mass + pos;
            c_mass_count = c_mass_count + 1;
        }
        if pos.distance(v_pos) < params.rule2_distance {
            col_vel = col_vel - (pos - v_pos);
        }
        if pos.distance(v_pos) < params.rule3_distance {
            c_vel = c_vel + vel;
            c_vel_count = c_vel_count + 1;
        }

        i = i + 1;
    }
    if c_mass_count > 0 {
        c_mass = c_mass * (1.0 / c_mass_count as f32) - v_pos;
    }
    if c_vel_count > 0 {
        c_vel = c_vel * (1.0 / c_vel_count as f32);
    }

    v_vel = v_vel
        + (c_mass * params.rule1_scale)
        + (col_vel * params.rule2_scale)
        + (c_vel * params.rule3_scale);

    // clamp velocity for a more pleasing simulation
    v_vel = v_vel.normalize() * v_vel.clamp_length(0.0, 0.1);

    // kinematic update
    v_pos = v_pos + (v_vel * params.delta_t);

    // Wrap around boundary
    if v_pos.x < -1.0 {
        v_pos.x = 1.0;
    }
    if v_pos.x > 1.0 {
        v_pos.x = -1.0;
    }
    if v_pos.y < -1.0 {
        v_pos.y = 1.0;
    }
    if v_pos.y > 1.0 {
        v_pos.y = -1.0;
    }

    // Write back
    *particles_dst.particles[index].pos = *v_pos;
    *particles_dst.particles[index].vel = *v_vel;
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vert_id: i32,
    particle_pos: Vec2,
    particle_vel: Vec2,
    position: Vec2,
    #[spirv(position)] builtin_pos: &mut Vec4,
) {
    let angle = -particle_vel.x.atan2(particle_vel.y);
    let pos = vec2(
        position.x * angle.cos() - position.y * angle.sin(),
        position.x * angle.sin() + position.y * angle.cos(),
    );
    *builtin_pos = vec4(pos.x + particle_pos.x, pos.y + particle_pos.y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4) {
    *output = vec4(1.0, 0.0, 0.0, 1.0);
}
