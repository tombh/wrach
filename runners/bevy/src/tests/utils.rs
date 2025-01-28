//! Rust interface to Wrach simulations

use bevy::prelude::PluginGroup as _;
use bevy::{app::App, winit::WinitPlugin, DefaultPlugins};

use crate::{Particle, WrachConfig, WrachPlugin, WrachState};

/// Main struct for Wrach physics simulations
pub struct WrachTestAPI {
    /// An instance of a Bevy app, already setup for Wrach
    pub app: App,
}

impl WrachTestAPI {
    /// Instantiate
    #[must_use]
    #[inline]
    pub fn new(config: WrachConfig) -> Self {
        let mut wrach = Self { app: App::new() };

        let plugin = WrachPlugin { config };
        wrach
            .app
            .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
            .add_plugins(plugin);
        wrach.app.finish();
        wrach.app.cleanup();
        wrach
    }

    /// Run a single tick/frame of the simulation.
    #[inline]
    pub fn tick(&mut self) {
        self.app.update();
    }

    /// Run until we get our first data.
    #[inline]
    pub fn tick_until_first_data(&mut self) {
        for _ in 0..5_u32 {
            self.tick();
            let data = &self.get_simulation_state().packed_data;
            if data.positions.first().is_some() {
                break;
            }
        }
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
