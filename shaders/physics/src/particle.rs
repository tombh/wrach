#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::neighbours;
use crate::world;
use crate::wrach_glam::glam::Vec2;

use core::f32::consts::PI;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const INFLUENCE_FACTOR: u32 = 3;
    } else {
        pub const INFLUENCE_FACTOR: u32 = 1;
    }
}

pub const PIXEL_SIZE: f32 = 2.0 / world::MAP_WIDTH as f32;
pub const PARTICLE_RADIUS: f32 = PIXEL_SIZE / 2.0;
const DEFAULT_VISCOSITY: f32 = 0.0;
pub const DEFAULT_NUM_SOLVER_SUBSTEPS: usize = 10;
const TIME_STEP: f32 = DEFAULT_NUM_SOLVER_SUBSTEPS as f32 / 1000.0;
const UNILATERAL: bool = true;
const PARTICLE_DIAMETER: f32 = 2.0 * PARTICLE_RADIUS;
const REST_DENSITY: f32 = 1.0 / (PARTICLE_DIAMETER * PARTICLE_DIAMETER);
pub const PARTICLE_INFLUENCE: f32 = INFLUENCE_FACTOR as f32 * PARTICLE_RADIUS; // kernel radius
const H2: f32 = PARTICLE_INFLUENCE * PARTICLE_INFLUENCE;
const KERNEL_SCALE: f32 = 4.0 / (PI * H2 * H2 * H2 * H2); // 2d poly6 (SPH based shallow water simulation)
const MAX_VEL: f32 = 0.5 * PARTICLE_RADIUS;

const DT: f32 = TIME_STEP / DEFAULT_NUM_SOLVER_SUBSTEPS as f32;

pub type ParticleID = u32;

pub type ParticlePosition = Vec2;
pub type ParticlePositions = [ParticlePosition; world::NUM_PARTICLES];

pub type ParticleVelocity = Vec2;
pub type ParticleVelocities = [ParticleVelocity; world::NUM_PARTICLES];

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct ParticlePropogation {
    pub previous: Vec2,
    pub lambda: f32,
}
pub type ParticlePropogations = [ParticlePropogation; world::NUM_PARTICLES];

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Particle {
    pub id: ParticleID,
    pub position: ParticlePosition,
    pub previous: ParticlePosition,
    pub pre_fluid_position: Vec2,
    pub velocity: ParticleVelocity,
    pub lambda: f32,
}

impl Particle {
    pub fn new(particle: Particle) -> Particle {
        let mut particle = Particle {
            id: particle.id,
            position: particle.position,
            previous: particle.previous,
            pre_fluid_position: Vec2::ZERO,
            velocity: particle.velocity,
            lambda: particle.lambda,
        };
        particle.update_pre_fluid_position();
        particle
    }

    fn to_current_particle(self, neighbours: neighbours::NeighbouringParticles) -> CurrentParticle {
        CurrentParticle::new(self, neighbours)
    }

    // TODO: explain
    pub fn predict(&mut self) {
        self.velocity += world::G * DT;
        self.previous = self.position;
        self.position += self.velocity * DT;
    }

    // Because maths is much cheaper than memory access
    fn update_pre_fluid_position(&mut self) {
        self.pre_fluid_position = self.previous + (self.velocity * DT);
    }

    pub fn compute(&mut self, neighbours: neighbours::NeighbouringParticles) -> Particle {
        let mut current_particle = self.to_current_particle(neighbours);
        current_particle.compute();
        current_particle.particle
    }

    pub fn propogate(&mut self, neighbours: neighbours::NeighbouringParticles) -> Particle {
        let mut current_particle = self.to_current_particle(neighbours);
        current_particle.propogate();
        current_particle.particle
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct CurrentParticle {
    pub particle: Particle,
    neighbours: neighbours::NeighbouringParticles,
}

impl CurrentParticle {
    pub fn new(
        particle: Particle,
        neighbours: neighbours::NeighbouringParticles,
    ) -> CurrentParticle {
        CurrentParticle {
            particle: Particle::new(particle),
            neighbours,
        }
    }

    // THIS IS HORRIBLE
    // After a refactor we suddenly needed this _second_ guard clause (as well
    // as the one at the main entry point). It very much seems like undefined
    // behaviour that execution for IDs above the number of particles should ever
    // reach here, but, well it does, and I have absolutely no idea why, it's soo
    // wrong 😭
    fn is_id_unsafe_hack(&self) -> bool {
        self.particle.id >= world::NUM_PARTICLES as ParticleID
    }

    pub fn compute(&mut self) {
        self.solve_boundaries();
        self.particle.predict();
        self.solve_fluid();
    }

    fn solve_boundaries(&mut self) {
        let wall = 0.95;
        let p = &mut self.particle.position;
        p.y = p.y.clamp(-wall, wall);
        p.x = p.x.clamp(-wall, wall);
    }

    fn solve_fluid(&mut self) {
        if self.is_id_unsafe_hack() {
            return;
        }
        let mut rho = 0.0;
        let mut sum_grad2 = 0.0;
        let mut grad_i = Vec2::ZERO;
        for i in 0..self.neighbours.length() {
            let mut neighbour = self.neighbours.get_neighbour(i);
            if neighbour.id != self.particle.id {
                neighbour.predict();
            }
            // TODO reuse the length from grid search?
            let mut n = neighbour.position - self.particle.position;
            let r = n.length();
            // normalize
            if r > 0.0 {
                n /= r;
            }
            let r2 = r * r;
            let w = H2 - r2;
            rho += KERNEL_SCALE * w * w * w;
            let grad = (KERNEL_SCALE * 3.0 * w * w * (-2.0 * r)) / REST_DENSITY;
            grad_i -= n * grad;
            sum_grad2 += grad * grad;
        }

        let c = rho / REST_DENSITY - 1.0;
        if UNILATERAL && c < 0.0 {
            self.particle.lambda = 0.0;
            return;
        }

        sum_grad2 += grad_i.length_squared();
        let lambda = -c / (sum_grad2 + 0.0001);
        self.particle.position += lambda * grad_i;
        self.particle.lambda = lambda;
    }

    fn propogate(&mut self) {
        if self.is_id_unsafe_hack() {
            return;
        }
        let mut tmp_position = self.particle.pre_fluid_position;
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
            if neighbour.id == self.particle.id {
                continue;
            }
            // TODO reuse the length from grid search?
            let mut n = self.particle.pre_fluid_position - neighbour.pre_fluid_position;
            // let mut n = self.particle.position - neighbour.position;
            let r = n.length();
            // normalize
            if r > 0.0 {
                n /= r;
            }
            let r2 = r * r;
            let w = H2 - r2;
            let grad = (KERNEL_SCALE * 3.0 * w * w * (-2.0 * r)) / REST_DENSITY;
            tmp_position += neighbour.lambda * (n * grad);
        }
        self.particle.position = tmp_position;

        // derive velocities
        let mut v = self.particle.position - self.particle.previous;
        let vel = v.length();

        // CFL
        let max_velocity = if self.particle.id == 450 {
            MAX_VEL * 2.0
        } else {
            MAX_VEL
        };
        if vel > max_velocity {
            v *= max_velocity / vel;
            self.particle.position = self.particle.previous + v;
        }
        self.particle.velocity = v / DT;
        self.apply_viscosity();
    }

    fn apply_viscosity(&mut self) {
        let mut avg_vel = Vec2::ZERO;
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
            if neighbour.id == self.particle.id {
                continue;
            }
            avg_vel += neighbour.velocity;
        }
        avg_vel /= self.neighbours.length() as f32;
        let delta = avg_vel - self.particle.velocity;
        self.particle.velocity += DEFAULT_VISCOSITY * delta;
    }
}
