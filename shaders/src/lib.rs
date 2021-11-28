#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr, asm),
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
pub mod workgroup;
pub mod world;
pub mod wrach_glam;

use wrach_glam::glam::{vec4, UVec3, Vec2, Vec4};

pub const SPIRV_TRUE: u32 = 1;
pub const SPIRV_FALSE: u32 = 0;

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(workgroup_id)] workgroup_id: UVec3,
    #[spirv(local_invocation_id)] local_id: UVec3,
    #[spirv(push_constant)] _params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::ParticlesGlobal,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut particle::ParticlesGlobal,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pixel_grid: &mut neighbours::PixelGridGlobal,
) {
    let ids = compute::GPUIdentifiers {
        global: global_id.x,
        workgroup: workgroup_id.x,
        local: local_id.x,
    };
    if ids.global >= world::NUM_PARTICLES as u32 {
        return;
    }
    neighbours::NeighbouringParticles::place_particle_in_pixel_grid(ids.global_id_to_particle_id(), particles_src, pixel_grid);
}

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(workgroup_id)] workgroup_id: UVec3,
    #[spirv(local_invocation_id)] local_id: UVec3,
    #[spirv(push_constant)] _params: &compute::Params,
    // #[spirv(workgroup)] workgroup_data: &mut workgroup::WorkGroupData,
    #[spirv(workgroup)] workgroup_data2: &mut [u32; workgroup::THREADS as usize],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] _particles_src: &mut particle::ParticlesGlobal,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut particle::ParticlesGlobal,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] _pixel_grid: &mut neighbours::PixelGridGlobal,
) {
    let ids = compute::GPUIdentifiers {
        global: global_id.x,
        workgroup: workgroup_id.x,
        local: local_id.x,
    };

    // if ids.global == 0 {
    //     let mut i = 0;
    //     while i < 8 {
    //         workgroup_data.neighbours[ids.workgroup as usize][i].id = 0;
    //         i += 1;
    //     }
    // }


    for i in 0..625 {
        if i as u32 % workgroup::THREADS == ids.local as u32 {
            workgroup_data2[i] = ids.local;
        }
    }

    unsafe { spirv_std::arch::workgroup_memory_barrier(); }

    // compute::entry(ids, params, particles_src, particles_dst, workgroup_data, pixel_grid);

//     particles_dst[id as usize].color = vec4(1.0, 1.0, 1.0, 0.0);
//     if id == 450 {
//         // let cp = particles_src[id as usize];
//         // for i in 0..world::NUM_PARTICLES {
//         //     let np = particles_src[i];
//         //     let distance = np.position.distance(cp.position);
//         //     if distance < particle::PARTICLE_INFLUENCE {
//         //         _particles_dst[i].color = vec4(1.0, 0.0, 0.0, 0.0);
//         //     }
//         // }
//         //
//         local_particles[0] = particles_dst[id as usize];
//         let mut neighbours =
//         neighbours::NeighbouringParticles::find(id as particle::ParticleID, map, local_particles);
//         for n in 0..neighbours.length() {
//             let np = neighbours.get_neighbour(n);
//             particles_dst[np.id as usize].color = vec4(1.0, 0.0, 0.0, 0.0);
//
//         }
//         particles_dst[id as usize].color = vec4(0.0, 1.0, 0.0, 0.0);
//     }
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
