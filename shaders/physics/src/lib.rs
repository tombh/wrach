#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]
#![allow(clippy::needless_range_loop)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

pub mod compute;
pub mod neighbours;
pub mod particle;
pub mod world;
pub mod wrach_glam;

use wrach_glam::glam::UVec3;

#[allow(clippy::too_many_arguments)]
#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] iduv: UVec3,
    #[spirv(push_constant)] params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] positions_src: &mut particle::ParticlePositions,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] positions_dst: &mut particle::ParticlePositions,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] velocities_src: &mut particle::ParticleVelocities,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] velocities_dst: &mut particle::ParticleVelocities,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] propogations: &mut particle::ParticlePropogations,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] map: &mut neighbours::PixelMapBasic,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] neighbourhood_ids: &mut neighbours::NeighbourhoodIDsBuffer,
) {
    let id = iduv.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as particle::ParticleID {
        return;
    }
    compute::entry(
        iduv,
        params,
        positions_src,
        positions_dst,
        velocities_src,
        velocities_dst,
        propogations,
        map,
        neighbourhood_ids,
        0
    );
}

#[allow(clippy::too_many_arguments)]
#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn post_main_cs(
    #[spirv(global_invocation_id)] iduv: UVec3,
    #[spirv(push_constant)] params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] positions_src: &mut particle::ParticlePositions,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] positions_dst: &mut particle::ParticlePositions,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] velocities_src: &mut particle::ParticleVelocities,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] velocities_dst: &mut particle::ParticleVelocities,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] propogations: &mut particle::ParticlePropogations,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] map: &mut neighbours::PixelMapBasic,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] neighbourhood_ids: &mut neighbours::NeighbourhoodIDsBuffer,
) {
    let id = iduv.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as particle::ParticleID {
        return;
    }
    compute::entry(
        iduv,
        params,
        positions_src,
        positions_dst,
        velocities_src,
        velocities_dst,
        propogations,
        map,
        neighbourhood_ids,
        1
    );

    if id == 450 {
        // let mut neighbours =
        // neighbours::NeighbouringParticles::recruit(
        //     id as particle::ParticleID,
        //     particles_src,
        //     neighbourhood_ids
        // );
        // for n in 0..neighbours.length() {
        //     let np = neighbours.get_neighbour(n);
        //     particles_dst[np.id as usize].color = vec4(1.0, 0.0, 0.0, 0.0);
        //
        // }
        // particles_dst[id as usize].color = vec4(0.0, 1.0, 0.0, 0.0);

        let delta = 0.03;
        if params.up > 0 {
            velocities_dst[id as usize].y += delta;
        }

        if params.down > 0 {
            velocities_dst[id as usize].y -= delta;
        }

        if params.left > 0 {
            velocities_dst[id as usize].x -= delta;
        }

        if params.right > 0 {
            velocities_dst[id as usize].x += delta;
        }

    }
}
