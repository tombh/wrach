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

use wrach_glam::glam::UVec3;

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn pre_main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] _particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] map: &mut neighbours::PixelMapBasic,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] _neighbourhood_ids: &mut neighbours::NeighbourhoodIDsBuffer,
) {
    let id = id.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as u32 {
        return;
    }
    neighbours::NeighbouringParticles::place_particle_in_pixel(id, particles_src, map);
}

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] iduv: UVec3,
    #[spirv(push_constant)] params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] map: &mut neighbours::PixelMapBasic,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] neighbourhood_ids: &mut neighbours::NeighbourhoodIDsBuffer,
) {
    let id = iduv.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as u32 {
        return;
    }
    neighbours::NeighbouringParticles::find(
        id as particle::ParticleID,
        map,
        particles_src,
        neighbourhood_ids,
    );
    compute::entry(iduv, params, particles_src, particles_dst, map, neighbourhood_ids, 0);
}

#[rustfmt::skip]
#[spirv(compute(threads(128)))]
pub fn post_main_cs(
    #[spirv(global_invocation_id)] iduv: UVec3,
    #[spirv(push_constant)] params: &compute::Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_src: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] particles_dst: &mut particle::Particles,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] map: &mut neighbours::PixelMapBasic,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] neighbourhood_ids: &mut neighbours::NeighbourhoodIDsBuffer,
) {
    let id = iduv.x as particle::ParticleID;
    if id >= world::NUM_PARTICLES as u32 {
        return;
    }
    compute::entry(iduv, params, particles_src, particles_dst, map, neighbourhood_ids, 1);
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
            particles_dst[id as usize].velocity.y += delta;
        }

        if params.down > 0 {
            particles_dst[id as usize].velocity.y -= delta;
        }

        if params.left > 0 {
            particles_dst[id as usize].velocity.x -= delta;
        }

        if params.right > 0 {
            particles_dst[id as usize].velocity.x += delta;
        }

    }
}
