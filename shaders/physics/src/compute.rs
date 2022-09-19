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

#[allow(clippy::too_many_arguments)]
pub fn entry(
    id: UVec3,
    _params: &Params,
    positions_src: &mut particle::ParticlePositions,
    positions_dst: &mut particle::ParticlePositions,
    velocities_src: &mut particle::ParticleVelocities,
    velocities_dst: &mut particle::ParticleVelocities,
    propogations: &mut particle::ParticlePropogations,
    map: &mut neighbours::PixelMapBasic,
    neighbourhood_ids_buffer: &mut neighbours::NeighbourhoodIDsBuffer,
    stage: u32,
) {
    let id = id.x as usize;
    if id >= world::NUM_PARTICLES {
        return;
    }

    let mut particle = particle::Particle::default();

    match stage {
        // Particle:
        //   Reads: 2xVec2,
        //   Writes: 3xVec2, 1xf32
        // Neighbours:
        //   Reads: ~32xu32 (IDs), ~18xVec2 (data)
        //   Writes: 0
        // Total: 712b
        0 => {
            let neighbourhood_ids = neighbours::NeighbouringParticles::find(
                id as particle::ParticleID,
                map,
                positions_src,
                neighbourhood_ids_buffer,
            );
            let neighbours = neighbours::NeighbouringParticles::recruit_from_ids(
                id as particle::ParticleID,
                positions_src,
                velocities_src,
                propogations,
                neighbourhood_ids,
                stage,
            );
            particle = particle::Particle::new(particle::Particle {
                id: id as particle::ParticleID,
                position: positions_src[id],
                velocity: velocities_src[id],
                ..Default::default()
            });
            particle = particle.compute(neighbours);
            positions_dst[id] = particle.position;
            velocities_dst[id] = particle.velocity;
            propogations[id] = particle::ParticlePropogation {
                previous: particle.previous,
                lambda: particle.lambda,
            }
        }

        // Particle:
        //   Reads: 2xVec2, 1xf32
        //   Writes: 3xVec2
        // Neighbours:
        //   Reads: ~9xu32 (IDs), ~27xVec2 (data), ~9xf32 (data)
        //   Writes: 0
        // Total: 624b
        1 => {
            let neighbours = neighbours::NeighbouringParticles::recruit_from_global(
                id as particle::ParticleID,
                positions_src,
                velocities_src,
                propogations,
                neighbourhood_ids_buffer,
                stage,
            );
            let propogation = propogations[id];
            particle = particle::Particle::new(particle::Particle {
                id: id as particle::ParticleID,
                velocity: velocities_src[id],
                previous: propogation.previous,
                lambda: propogation.lambda,
                ..Default::default()
            });
            particle = particle.propogate(neighbours);
            positions_dst[id] = particle.position;
            // velocities_dst[id] = particle.velocity;
            neighbours::NeighbouringParticles::place_particle_in_pixel(
                id as particle::ParticleID,
                particle.position,
                map,
            );
        }

        _ => _ = particle,
    }
}
