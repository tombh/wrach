#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::neighbours;
use crate::world;
use crate::wrach_glam::glam::{vec2, vec4, Vec2, Vec4};

use core::f32::consts::PI;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const INFLUENCE_FACTOR: u32 = 10;
    } else {
        pub const INFLUENCE_FACTOR: u32 = 1;
    }
}
const PARTICLE_RADIUS: f32 = 0.01;
const DEFAULT_VISCOSITY: f32 = 0.0;
const TIME_STEP: f32 = 0.01;
const DEFAULT_NUM_SOLVER_SUBSTEPS: usize = 1;
const UNILATERAL: bool = true;
const PARTICLE_DIAMETER: f32 = 2.0 * PARTICLE_RADIUS;
const REST_DENSITY: f32 = 1.0 / (PARTICLE_DIAMETER * PARTICLE_DIAMETER);
const PARTICLE_INFLUENCE: f32 = 3.0 as f32 * PARTICLE_RADIUS; // kernel radius
const H2: f32 = PARTICLE_INFLUENCE * PARTICLE_INFLUENCE;
const KERNEL_SCALE: f32 = 4.0 / (PI * H2 * H2 * H2 * H2); // 2d poly6 (SPH based shallow water simulation)
const MAX_VEL: f32 = 0.4 * PARTICLE_RADIUS;

const DT: f32 = TIME_STEP / DEFAULT_NUM_SOLVER_SUBSTEPS as f32;

pub type ParticleID = u32;
pub const NO_PARTICLE_ID: ParticleID = 0;

// Field order matters!! Because of std140, wgpu, spirv, etc
#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct ParticleBasic {
    pub color: Vec4,
    pub position: Vec2,
    pub velocity: Vec2,
    pub gradient: Vec2,
}

impl ParticleBasic {
    fn to_current_particle(
        &self,
        id: ParticleID,
        neighbours: neighbours::NeighbouringParticles,
    ) -> CurrentParticle {
        CurrentParticle::new(id, *self, neighbours)
    }
    pub fn update(&mut self, id: ParticleID, neighbours: neighbours::NeighbouringParticles) {
        let mut current_particle = self.to_current_particle(id, neighbours);
        current_particle.compute();
        self.position = current_particle.particle.position;
        self.velocity = current_particle.particle.velocity;
        self.gradient = current_particle.particle.gradient;
        self.color = current_particle.particle.color;
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
pub struct Particle {
    pub id: ParticleID,
    pub position: Vec2,
    pub previous: Vec2,
    pub velocity: Vec2,
    pub gradient: Vec2,
    pub color: Vec4,
}

impl Particle {
    pub fn new(id: ParticleID, particle_basic: ParticleBasic) -> Particle {
        Particle {
            id,
            position: particle_basic.position,
            velocity: particle_basic.velocity,
            gradient: particle_basic.gradient,
            color: particle_basic.color,
            previous: Default::default(),
        }
    }
}

pub trait ParticleaAsPixel {
    fn pixel_position(&self) -> Vec2 {
        vec2(0.0, 0.0)
    }
    fn scale(&self, position: f32, scale: u32) -> f32 {
        ((position + 1.0) / 2.0) * (scale - 1) as f32
    }
}

// TODO: is there a way to de-duplicate these?
impl ParticleaAsPixel for Particle {
    fn pixel_position(&self) -> Vec2 {
        vec2(
            self.scale(self.position.x, world::MAP_WIDTH),
            self.scale(self.position.y, world::MAP_HEIGHT),
        )
    }
}
impl ParticleaAsPixel for ParticleBasic {
    fn pixel_position(&self) -> Vec2 {
        vec2(
            self.scale(self.position.x, world::MAP_WIDTH),
            self.scale(self.position.y, world::MAP_HEIGHT),
        )
    }
}

pub type Particles = [ParticleBasic; world::NUM_PARTICLES];

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct CurrentParticle {
    pub particle: Particle,
    pub neighbours: neighbours::NeighbouringParticles,
}

impl CurrentParticle {
    pub fn new(
        id: ParticleID,
        particle_basic: ParticleBasic,
        neighbours: neighbours::NeighbouringParticles,
    ) -> CurrentParticle {
        CurrentParticle {
            particle: Particle::new(id, particle_basic),
            neighbours,
        }
    }

    pub fn compute(&mut self) {
        for _ in 0..DEFAULT_NUM_SOLVER_SUBSTEPS {
            // predict
            self.particle.velocity += world::G * DT;
            self.particle.previous = self.particle.position;
            self.particle.position += self.particle.velocity * DT;

            // solve
            self.solve_fluid();

            // derive velocities
            let mut v = self.particle.position - self.particle.previous;
            let vel = v.length();

            // CFL
            if vel > MAX_VEL {
                v *= MAX_VEL / vel;
                self.particle.position = self.particle.previous + v;
            }
            self.particle.velocity = v / DT;
            self.apply_viscosity();
            self.solve_boundaries();
        }
    }

    fn solve_boundaries(&mut self) {
        let p = &mut self.particle.position;
        let v = &mut self.particle.velocity;

        if p.y <= -1.0 {
            v.y = -v.y;
        }
        if p.y >= 1.0 {
            v.y = -v.y;
        }
        if p.x <= -1.0 {
            v.x = -v.x;
        }
        if p.x >= 1.0 {
            v.x = -v.x;
        }

        //         p.y = p.y.clamp(-1.0, 1.0);
        //
        //         // left and right bounds
        //         p.x = p.x.clamp(-1.0, 1.0);
    }

    fn solve_fluid(&mut self) {
        let mut rho = 0.0;
        let mut sum_grad2 = 0.0;
        let mut grad_i = Vec2::ZERO;
        self.particle.color = vec4(1.0, 1.0, 1.0, 0.0);
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
            if neighbour.id == self.particle.id {
                continue;
            }
            let mut n = neighbour.position - self.particle.position;
            let r = n.length();
            // normalize
            if r != 0.0 {
                n /= r;
            }
            if r <= PARTICLE_INFLUENCE {
                let r2 = r * r;
                let w = H2 - r2;
                rho += KERNEL_SCALE * w * w * w;
                let grad = (KERNEL_SCALE * 3.0 * w * w * (-2.0 * r)) / REST_DENSITY;
                self.particle.gradient = n * grad;
                grad_i -= n * grad;
                sum_grad2 += grad * grad;
            }
        }

        let c = rho / REST_DENSITY - 1.0;
        if UNILATERAL && c < 0.0 {
            return;
        }

        self.particle.color = vec4(0.0, 1.0, 0.0, 0.0);

        sum_grad2 += grad_i.length_squared();
        let lambda = -c / (sum_grad2 + 0.0001);
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
            let diff: Vec2;
            if neighbour.id == self.particle.id {
                diff = lambda * grad_i;
            } else {
                // diff = lambda * neighbour.gradient;
                diff = vec2(0.0, 0.0);
            }
            // TODO: applies to neighbour!!!!!!!!!!!!!
            self.particle.position += diff;
        }
    }

    fn apply_viscosity(&mut self) {
        let mut avg_vel = Vec2::ZERO;
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
            avg_vel += neighbour.velocity;
        }
        avg_vel /= self.neighbours.length() as f32;
        let delta = avg_vel - self.particle.velocity;
        self.particle.velocity += DEFAULT_VISCOSITY * delta;
    }
}
