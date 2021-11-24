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
#[spirv(compute(threads(128)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] map: &mut neighbours::PixelMapBasic,
) {
    // Prevents the work item continuing until all work items in workgroup have reached it
    // unsafe {
    //     use spirv_std::memory::{Scope, Semantics};
    //     spirv_std::arch::control_barrier::<
    //         { Scope::Workgroup as u32 },
    //         { Scope::Workgroup as u32 },
    //         { Semantics::NONE.bits() },
    //     >();
    // }
    let id = id.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as u32 {
        return;
    }
    neighbours::NeighbouringParticles::place_particle_in_pixel(id, particles_src, map);
}

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] map: &mut neighbours::PixelMapBasic,
) {
    compute::entry(id, params, particles_src, particles_dst, map, params.stage);
    let id = id.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as u32 {
        return;
    }
    particles_dst[id as usize].color = vec4(1.0, 1.0, 1.0, 0.0);
    if id == 450 {
        // let cp = particles_src[id as usize];
        // for i in 0..world::NUM_PARTICLES {
        //     let np = particles_src[i];
        //     let distance = np.position.distance(cp.position);
        //     if distance < particle::PARTICLE_INFLUENCE {
        //         _particles_dst[i].color = vec4(1.0, 0.0, 0.0, 0.0);
        //     }
        // }
        //
        let mut neighbours =
        neighbours::NeighbouringParticles::find(id as particle::ParticleID, map, particles_src);
        for n in 0..neighbours.length() {
            let np = neighbours.get_neighbour(n);
            particles_dst[np.id as usize].color = vec4(1.0, 0.0, 0.0, 0.0);

        }
        particles_dst[id as usize].color = vec4(0.0, 1.0, 0.0, 0.0);
    }
}

// Called for every index of a vertex, there are 6 in a square, because a square
// is made up from 2 triangles
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vert_id: i32,
    #[spirv(position)] screen_position: &mut Vec4,
    #[spirv(push_constant)] _params: &compute::Params,
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
pub fn main_fs(#[spirv(push_constant)] _params: &compute::Params, input: Vec4, output: &mut Vec4) {
    *output = vec4(input.x, input.y, input.z, input.w);
}
