use crate::wrach_glam::glam::UVec3;

use crate::neighbours;
use crate::particle;
use crate::world;

pub struct SimParams {
    _delta_t: f32,
    _rule1_distance: f32,
    _rule2_distance: f32,
    _rule3_distance: f32,
    _rule1_scale: f32,
    _rule2_scale: f32,
    _rule3_scale: f32,
}

pub fn entry(
    id: UVec3,
    _params: &SimParams,
    particles_src: &mut particle::Particles,
    particles_dst: &mut particle::Particles,
    pixel_map: &world::PixelMapBasic,
) {
    let id = id.x as usize;
    let neighbours = neighbours::NeighbouringParticles::find(
        id as particle::ParticleID,
        pixel_map,
        particles_src,
    );
    particles_src[id].update(id as particle::ParticleID, neighbours);
    particles_dst[id] = particles_src[id];
}
