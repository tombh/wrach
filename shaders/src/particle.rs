#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::neighbours;
use crate::workgroup;
use crate::world;
use crate::wrach_glam::glam::{vec2, Vec2, Vec4};

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

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
pub struct ParticleIDGlobal {
    pub id: u32,
}

impl ParticleIDGlobal {
    pub fn null() -> u32 {
        u32::MAX
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Copy, Clone)]
pub struct ParticleIDLocal {
    pub id: u32,
}

impl ParticleIDLocal {
    pub fn null() -> u32 {
        u32::MAX
    }
}

// Field order matters!! Because of std140, wgpu, spirv, etc
#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct ParticleGlobal {
    pub color: Vec4,
    pub position: Vec2,
    pub velocity: Vec2,
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Copy, Clone)]
pub struct ParticleLocal {
    pub id_global: ParticleIDGlobal,
    pub color: Vec4,
    pub position: Vec2,
    pub previous: Vec2,
    pub pre_fluid_position: Vec2,
    pub velocity: Vec2,
    pub lambda: f32,
}

impl ParticleLocal {
    pub fn new(id: ParticleIDGlobal, particle_global: ParticleGlobal) -> ParticleLocal {
        ParticleLocal {
            id_global: id,
            position: particle_global.position,
            velocity: particle_global.velocity,
            color: particle_global.color,
            lambda: 0.0,
            pre_fluid_position: vec2(0.0, 0.0),
            previous: vec2(0.0, 0.0),
        }
    }

    pub fn to_global(&self) -> ParticleGlobal {
        ParticleGlobal {
            position: self.position,
            velocity: self.velocity,
            color: self.color,
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
impl ParticleaAsPixel for ParticleLocal {
    fn pixel_position(&self) -> Vec2 {
        vec2(
            self.scale(self.position.x, neighbours::PIXEL_GRID_GLOBAL_COLS),
            self.scale(self.position.y, neighbours::PIXEL_GRID_GLOBAL_ROWS),
        )
    }
}
impl ParticleaAsPixel for ParticleGlobal {
    fn pixel_position(&self) -> Vec2 {
        vec2(
            self.scale(self.position.x, neighbours::PIXEL_GRID_GLOBAL_COLS),
            self.scale(self.position.y, neighbours::PIXEL_GRID_GLOBAL_ROWS),
        )
    }
}

pub type ParticlesGlobal = [ParticleGlobal; world::NUM_PARTICLES];

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct CurrentParticle {
    pub particle: ParticleLocal,
    thread_id: usize,
}

impl CurrentParticle {
    pub fn new(thread_id: usize, particle_local: ParticleLocal) -> CurrentParticle {
        CurrentParticle {
            thread_id,
            particle: particle_local,
        }
    }

    pub fn predict(&mut self) {
        self.particle.velocity += world::G * DT;
        self.particle.previous = self.particle.position;
        self.particle.position += self.particle.velocity * DT;
        self.solve_boundaries();
        self.particle.pre_fluid_position = self.particle.position;
    }

    pub fn compute(&mut self, workgroup_data: &mut workgroup::WorkGroupData) {
        self.solve_fluid(workgroup_data);
    }

    fn get_neighbour(
        &mut self,
        workgroup_data: &workgroup::WorkGroupData,
        i: usize,
    ) -> ParticleIDLocal {
        let id = &workgroup_data.neighbours[self.thread_id][i + 1];
        ParticleIDLocal { id: id.id }
    }

    fn neighbours_count(&self, workgroup_data: &mut workgroup::WorkGroupData) -> usize {
        workgroup_data.neighbours[self.thread_id][0].id as usize
    }

    fn solve_boundaries(&mut self) {
        let wall = 0.95;
        let p = &mut self.particle.position;
        //
        p.y = p.y.clamp(-wall, wall);
        p.x = p.x.clamp(-wall, wall);
    }

    fn solve_fluid(&mut self, workgroup_data: &mut workgroup::WorkGroupData) {
        let mut rho = 0.0;
        let mut sum_grad2 = 0.0;
        let mut grad_i = Vec2::ZERO;
        for i in 0..self.neighbours_count(workgroup_data) {
            let neighbour_id = self.get_neighbour(workgroup_data, i);
            let neighbour = &workgroup_data.particles[neighbour_id.id as usize];
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

    pub fn propogate(&mut self, workgroup_data: &mut workgroup::WorkGroupData) {
        let mut tmp_position = self.particle.pre_fluid_position;
        for i in 0..self.neighbours_count(workgroup_data) {
            let neighbour_id = self.get_neighbour(workgroup_data, i);
            let neighbour = &workgroup_data.particles[neighbour_id.id as usize];
            if neighbour.id_global.id == self.particle.id_global.id {
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
        self.apply_viscosity(workgroup_data);
    }

    fn apply_viscosity(&mut self, workgroup_data: &mut workgroup::WorkGroupData) {
        let mut avg_vel = Vec2::ZERO;
        for i in 0..self.neighbours_count(workgroup_data) {
            let neighbour_id = self.get_neighbour(workgroup_data, i);
            let neighbour = &workgroup_data.particles[neighbour_id.id as usize];
            if neighbour.id_global.id == self.particle.id_global.id {
                continue;
            }
            avg_vel += neighbour.velocity;
        }
        avg_vel /= self.neighbours_count(workgroup_data) as f32;
        let delta = avg_vel - self.particle.velocity;
        self.particle.velocity += DEFAULT_VISCOSITY * delta;
    }
}
