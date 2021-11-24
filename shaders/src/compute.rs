#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::wrach_glam::glam::UVec3;

use crate::neighbours;
use crate::particle;
use crate::world;

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Params {
    pub stage: u32,
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
    grid: &neighbours::GridBasic,
    stage: u32,
) {
    let id = id.x as usize;
    if id >= world::NUM_PARTICLES {
        return;
    }
    let neighbours =
        neighbours::NeighbouringParticles::find(id as particle::ParticleID, grid, particles_src);
    match stage {
        0 => particles_src[id].predict(id as particle::ParticleID, neighbours),
        1 => particles_src[id].update(id as particle::ParticleID, neighbours),
        2 => particles_src[id].propogate(id as particle::ParticleID, neighbours),
        _ => (),
    }
    // TODO don't write to particles_src
    particles_dst[id] = particles_src[id];
}
