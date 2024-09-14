//! All the state for the simulation, both the physics itself and state for managing the simulation

use bevy::{math::Vec2, prelude::Resource};

/// All simulation state, exported for end users
#[derive(Default, Resource)]
#[allow(clippy::exhaustive_structs)]
pub struct WrachState {
    /// The maximum number of particles to simulate
    pub max_particles: u32,
    /// The particle positions
    pub positions: Vec<Vec2>,
    /// The particle velocities
    pub velocities: Vec<Vec2>,
    /// Data to send to the GPU, typically for CPU-side influence over the simulation
    pub gpu_uploads: Vec<GPUUpload>,
}

/// Data that the user wants to send to the GPU
#[derive(Default)]
#[allow(clippy::exhaustive_structs)]
pub struct GPUUpload {
    /// Particle positions to overwrite
    pub positions: Vec<Vec2>,
    /// Particle velocities
    pub velocities: Vec<Vec2>,
    // TODO: Add location at which to write
}

/// Wrach's representation of a particle. Probably will only ever be used for inserting.
#[allow(clippy::exhaustive_structs)]
pub struct Particle {
    /// Position of particle
    pub position: Position,
    /// Velocity of particle
    pub velocity: Velocity,
}

/// Wrach's type for particle position, could use some sort of `Vec2` instead
pub type Position = (f32, f32);
/// Wrach's type for particle velocity, could use some sort of `Vec2` instead
pub type Velocity = (f32, f32);

impl WrachState {
    /// Instantiate
    #[inline]
    #[must_use]
    pub fn new(max_particles: u32) -> Self {
        Self {
            max_particles,
            ..Default::default()
        }
    }

    /// Overwrites the simulation data from the first pixel to the size of the overwriting data
    #[inline]
    pub fn gpu_upload(&mut self, upload: GPUUpload) {
        self.gpu_uploads.push(upload);
    }

    /// Overwrites the simulation data from the first pixel to the size of the overwriting data
    #[inline]
    pub fn add_particles(&mut self, particles: Vec<Particle>) {
        let mut upload = GPUUpload {
            positions: self.positions.clone(),
            velocities: self.velocities.clone(),
        };

        for particle in particles {
            upload.positions.push(particle.position.into());
            upload.velocities.push(particle.velocity.into());
        }

        self.gpu_upload(upload);
    }
}
