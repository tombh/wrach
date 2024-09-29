//! Rust interface to Wrach simulations

// Apparently `pub use` is bad?
// https://rust-lang.github.io/rust-clippy/master/index.html#/pub_use
#![allow(clippy::pub_use)]

use bevy::prelude::PluginGroup;
use bevy::{app::App, winit::WinitPlugin, DefaultPlugins};

use crate::{Particle, WrachConfig, WrachPlugin, WrachState};

/// Main struct for Wrach physics simulations
#[allow(clippy::exhaustive_structs)]
pub struct WrachTestAPI {
    /// An instance of a Bevy app, already setup for Wrach
    pub app: App,
    /// All the positions of the particles
    pub positions: Vec<(f32, f32)>,
    /// All the velocities of the particles
    pub velocities: Vec<(f32, f32)>,
}

impl WrachTestAPI {
    /// Instantiate
    #[must_use]
    #[inline]
    pub fn new(config: WrachConfig) -> Self {
        let mut wrach = Self {
            app: App::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
        };

        let plugin = WrachPlugin { config };
        wrach
            .app
            .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
            .add_plugins(plugin);
        wrach.app.finish();
        wrach.app.cleanup();
        wrach
    }

    /// Run a single tick/frame of the simulation
    #[inline]
    pub fn tick(&mut self) {
        self.app.update();
        self.read_data();
    }

    /// Get data from the simulation
    // TODO: Check performance of this. Are we using the data directly? There's no copying going
    // on?
    #[inline]
    pub fn read_data(&mut self) {
        let state = self.app.world().resource::<WrachState>();

        self.positions = state
            .packed_data
            .positions
            .iter()
            .map(|particle| (particle.x, particle.y))
            .collect();

        self.velocities = state
            .packed_data
            .velocities
            .iter()
            .map(|particle| (particle.x, particle.y))
            .collect();
    }

    /// Add particles to the simulation
    #[inline]
    pub fn add_particles(&mut self, particles: Vec<Particle>) {
        let mut state = self.app.world_mut().resource_mut::<WrachState>();
        state.add_particles(particles);
    }

    /// Return the internal Bevy state for the simulation.
    #[inline]
    pub fn get_simulation_state(&self) -> &WrachState {
        self.app.world().resource::<WrachState>()
    }
}
