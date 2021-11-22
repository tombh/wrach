use crate::wrach_glam::glam::UVec3;

use crate::neighbours;
use crate::particle;
use crate::world;

pub struct SimParams {
    pub stage: u32,
    _rule1_distance: f32,
    _rule2_distance: f32,
    _rule3_distance: f32,
    _rule1_scale: f32,
    _rule2_scale: f32,
    _rule3_scale: f32,
}

// pub enum Stage {
//     Solve,
//     Propogate,
// }

pub fn entry(
    id: UVec3,
    _params: &SimParams,
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
