#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

use crate::neighbours;
use crate::world;
use crate::wrach_glam::glam::{Vec2, Vec4};

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
const MAX_VEL: f32 = 0.4 * PARTICLE_RADIUS;

const DT: f32 = TIME_STEP / DEFAULT_NUM_SOLVER_SUBSTEPS as f32;

pub type ParticleID = u32;

// Field order matters!! Because of std140, wgpu, spirv, etc
#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct ParticleBasic {
    pub color: Vec4,
    pub position: Vec2,
    pub previous: Vec2,
    pub pre_fluid_position: Vec2,
    pub velocity: Vec2,
    pub lambda: f32,
    pub grid_start_index: u32,
}

impl ParticleBasic {
    fn to_current_particle(
        &self,
        id: ParticleID,
        neighbours: neighbours::NeighbouringParticles,
    ) -> CurrentParticle {
        CurrentParticle::new(id, *self, neighbours)
    }

    pub fn predict(&mut self, id: ParticleID, neighbours: neighbours::NeighbouringParticles) {
        let mut current_particle = self.to_current_particle(id, neighbours);
        current_particle.predict();
        self.position = current_particle.particle.position;
        self.previous = current_particle.particle.previous;
        self.pre_fluid_position = current_particle.particle.pre_fluid_position;
        self.velocity = current_particle.particle.velocity;
        self.lambda = current_particle.particle.lambda;
        self.color = current_particle.particle.color;
        self.grid_start_index = current_particle.particle.grid_start_index();
    }

    pub fn update(&mut self, id: ParticleID, neighbours: neighbours::NeighbouringParticles) {
        let mut current_particle = self.to_current_particle(id, neighbours);
        current_particle.compute();
        self.position = current_particle.particle.position;
        self.previous = current_particle.particle.previous;
        self.pre_fluid_position = current_particle.particle.pre_fluid_position;
        self.velocity = current_particle.particle.velocity;
        self.lambda = current_particle.particle.lambda;
        self.color = current_particle.particle.color;
        self.grid_start_index = current_particle.particle.grid_start_index();
    }

    pub fn propogate(&mut self, id: ParticleID, neighbours: neighbours::NeighbouringParticles) {
        let mut current_particle = self.to_current_particle(id, neighbours);
        current_particle.propogate();
        self.position = current_particle.particle.position;
        self.previous = current_particle.particle.previous;
        self.pre_fluid_position = current_particle.particle.pre_fluid_position;
        self.velocity = current_particle.particle.velocity;
        self.lambda = current_particle.particle.lambda;
        self.color = current_particle.particle.color;
        self.grid_start_index = current_particle.particle.grid_start_index();
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
pub struct Particle {
    pub id: ParticleID,
    pub position: Vec2,
    pub previous: Vec2,
    pub pre_fluid_position: Vec2,
    pub velocity: Vec2,
    pub lambda: f32,
    pub color: Vec4,
}

impl Particle {
    pub fn new(id: ParticleID, particle_basic: ParticleBasic) -> Particle {
        Particle {
            id,
            position: particle_basic.position,
            previous: particle_basic.previous,
            pre_fluid_position: particle_basic.pre_fluid_position,
            velocity: particle_basic.velocity,
            lambda: particle_basic.lambda,
            color: particle_basic.color,
        }
    }
}

pub trait ParticleGridStartID {
    fn grid_start_index(&self) -> neighbours::GridStartID {
        0
    }
    fn scale(&self, position: f32, scale: u32) -> f32 {
        let scaled = ((position + 1.0) / 2.0) * scale as f32;
        scaled.clamp(0.0, scale as f32 - 1e-5)
    }
    fn grid_coords_from_particle_coords(&self) -> (u32, u32) {
        (0, 0)
    }
}

// TODO: is there a way to de-duplicate these?
impl ParticleGridStartID for Particle {
    fn grid_coords_from_particle_coords(&self) -> (u32, u32) {
        let x = self
            .scale(self.position.x, neighbours::GRIDS_PER_ROW)
            .floor() as u32;
        let y = self
            .scale(self.position.y, neighbours::GRIDS_PER_COL)
            .floor() as u32;
        (x, y)
    }
    fn grid_start_index(&self) -> neighbours::GridStartID {
        let (x, y) = self.grid_coords_from_particle_coords();
        neighbours::NeighbouringParticles::grid_coord_to_grid_start_index(x, y)
    }
}
impl ParticleGridStartID for ParticleBasic {
    fn grid_coords_from_particle_coords(&self) -> (u32, u32) {
        let x = self
            .scale(self.position.x, neighbours::GRIDS_PER_ROW)
            .floor() as u32;
        let y = self
            .scale(self.position.y, neighbours::GRIDS_PER_COL)
            .floor() as u32;
        (x, y)
    }
    fn grid_start_index(&self) -> neighbours::GridStartID {
        let (x, y) = self.grid_coords_from_particle_coords();
        neighbours::NeighbouringParticles::grid_coord_to_grid_start_index(x, y)
    }
}

pub type Particles = [ParticleBasic; world::NUM_PARTICLES];

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct CurrentParticle {
    pub particle: Particle,
    neighbours: neighbours::NeighbouringParticles,
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

    pub fn predict(&mut self) {
        self.particle.velocity += world::G * DT;
        self.particle.previous = self.particle.position;
        self.particle.position += self.particle.velocity * DT;
        self.solve_boundaries();
        self.particle.pre_fluid_position = self.particle.position;
    }

    pub fn compute(&mut self) {
        // solve
        self.solve_fluid();
    }

    fn solve_boundaries(&mut self) {
        let wall = 0.95;
        let p = &mut self.particle.position;

        p.y = p.y.clamp(-wall, wall);
        p.x = p.x.clamp(-wall, wall);
    }

    fn solve_fluid(&mut self) {
        let mut rho = 0.0;
        let mut sum_grad2 = 0.0;
        let mut grad_i = Vec2::ZERO;
        for i in 0..self.neighbours.length() {
            let neighbour = self.neighbours.get_neighbour(i);
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
        if vel > MAX_VEL {
            v *= MAX_VEL / vel;
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
