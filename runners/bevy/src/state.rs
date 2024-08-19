//! All the state for the simulation, both the physics itself and state for managing the simulation
use rand::Rng;

use bevy::{math::Vec2, prelude::Resource};

/// All simulation state, exported for end users
#[derive(Default, Resource)]
#[allow(clippy::exhaustive_structs)]
pub struct WrachState {
    /// The pixel positions
    pub positions: Vec<Vec2>,
    /// The pixel velocities
    pub velocities: Vec<Vec2>,
    /// Data to send to the GPU, typically for CPU-side influence over the simulation
    pub overwrite: Vec<Vec2>,
}

impl WrachState {
    /// Instantiate
    #[inline]
    #[must_use]
    pub fn new(size: i32) -> Self {
        let mut positions: Vec<Vec2> = Vec::new();
        for _ in 0_i32..size {
            let x = rand::thread_rng().gen_range(-1.0..1.0);
            let y = rand::thread_rng().gen_range(-1.0..1.0);
            positions.push(Vec2::new(x, y));
        }

        let mut velocities: Vec<Vec2> = Vec::new();
        let max_velocity = 0.001;
        for _ in 0_i32..size {
            let x = rand::thread_rng().gen_range(-max_velocity..max_velocity);
            let y = rand::thread_rng().gen_range(-max_velocity..max_velocity);
            velocities.push(Vec2::new(x, y));
        }

        Self {
            positions,
            velocities,
            overwrite: Vec::default(),
        }
    }

    /// Overwrites the simulation data from the first pixel to the size of the overwriting data
    #[inline]
    pub fn overwrite(&mut self, velocities: Vec<Vec2>) {
        self.overwrite = velocities;
    }
}
