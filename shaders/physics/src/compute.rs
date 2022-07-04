#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::particle::Particle;
use crate::particle::ParticleBasic;
use crate::wrach_glam::glam::UVec3;

use crate::neighbours;
use crate::particle;
use crate::world;

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Params {
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
}

// Crevice doesn't support enums, so maybe define this with bytemuck?
// pub enum Stage {
//     Solve,
//     Propogate,
// }

pub fn entry(
    id: UVec3,
    _params: &Params,
    particles_src: &mut particle::Particles,
    particles_dst: &mut particle::Particles,
    _grid: &neighbours::PixelMapBasic,
    neighbourhood_ids_buffer: &neighbours::NeighbourhoodIDsBuffer,
    stage: u32,
) {
    let id = id.x as usize;
    if id >= world::NUM_PARTICLES {
        return;
    }
    let neighbours = neighbours::NeighbouringParticles::recruit(
        id as particle::ParticleID,
        particles_src,
        neighbourhood_ids_buffer,
    );
    let particle = match stage {
        0 => particles_src[id].compute(id as particle::ParticleID, neighbours),
        1 => particles_src[id].propogate(id as particle::ParticleID, neighbours),
        _ => Particle::default(),
    };

    let particle_basic = ParticleBasic {
        lambda: particle.lambda,
        position: particle.position,
        previous: particle.previous,
        velocity: particle.velocity,
    };

    // TODO: shouldn't this be handled in the methods above?
    particles_dst[id] = particle_basic;
}
