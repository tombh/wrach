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

pub fn entry(
    id: UVec3,
    _params: &Params,
    positions_src: &mut particle::ParticlePositions,
    positions_dst: &mut particle::ParticlePositions,
    velocities_src: &mut particle::ParticleVelocities,
    velocities_dst: &mut particle::ParticleVelocities,
    propogations: &mut particle::ParticlePropogations,
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
        positions_src,
        velocities_src,
        propogations,
        neighbourhood_ids_buffer,
    );

    let mut particle = particle::Particle::default();

    match stage {
        // Reads: 2xVec2
        // Writes: 3xVec2, 1xf32
        0 => {
            particle = particle::Particle::new(particle::Particle {
                id: id as u32,
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

        // Reads: 2xVec2, 1xf32
        // Writes: 2xVec2
        1 => {
            let propogation = propogations[id];
            particle = particle::Particle::new(particle::Particle {
                id: id as u32,
                velocity: velocities_src[id],
                previous: propogation.previous,
                lambda: propogation.lambda,
                ..Default::default()
            });
            particle = particle.propogate(neighbours);
            positions_dst[id] = particle.position;
            velocities_dst[id] = particle.velocity;
        }

        _ => drop(particle),
    }
}
