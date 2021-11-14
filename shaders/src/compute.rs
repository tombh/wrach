#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

use crate::wrach_glam::glam::{vec2, UVec3};

pub use crate::particle::Particle;

pub const NUM_PARTICLES: usize = 1_00_000;

pub const MAP_WIDTH: u16 = 1600;
pub const MAP_HEIGHT: u16 = 800;
pub const MAP_SIZE: usize = MAP_WIDTH as usize * MAP_HEIGHT as usize;

pub type PixelMap = [u32; MAP_SIZE];

pub struct SimParams {
    _delta_t: f32,
    _rule1_distance: f32,
    _rule2_distance: f32,
    _rule3_distance: f32,
    _rule1_scale: f32,
    _rule2_scale: f32,
    _rule3_scale: f32,
}

pub struct Particles {
    pub particles: [Particle; NUM_PARTICLES],
}

pub fn entry(
    id: UVec3,
    _params: &SimParams,
    particles_src: &mut Particles,
    particles_dst: &mut Particles,
    pixel_map: &mut PixelMap,
) {
    let index = id.x as usize;

    let mut this_particle = particles_src.particles[index];

    let mut total_force = vec2(0.0, 0.0);

    let vacinity = 15.0;
    let x = this_particle.position.x.floor();
    let x_min = (x - vacinity) as usize;
    let x_max = (x + vacinity) as usize;
    let y = this_particle.position.y.floor();
    let y_min = (y - vacinity) as usize;
    let y_max = (y + vacinity) as usize;
    for y in y_min..y_max {
        for x in x_min..x_max {
            let coord = ((y * MAP_WIDTH as usize) + x) as usize;
            let other_particle_index = pixel_map[coord] as usize;
            if other_particle_index == index {
                continue;
            }
            if other_particle_index == 0 {
                continue;
            }
            let other_particle = particles_src.particles[other_particle_index];

            total_force += this_particle.force(other_particle);
        }
    }

    this_particle.velocity += total_force;
    this_particle.position += this_particle.velocity;

    this_particle.bounce_off_walls();

    // Write back
    particles_dst.particles[index].position = this_particle.position;
    particles_dst.particles[index].velocity = this_particle.velocity;
}
