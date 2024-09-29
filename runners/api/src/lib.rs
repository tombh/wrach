//! Rust interface to Wrach simulations

// Apparently `pub use` is bad?
// https://rust-lang.github.io/rust-clippy/master/index.html#/pub_use
#![allow(clippy::pub_use)]

use bevy::prelude::PluginGroup;
use bevy::{app::App, winit::WinitPlugin, DefaultPlugins};
use wrach_bevy::{WrachPlugin, WrachState};

pub use bevy::math::Vec2;
pub use wrach_bevy::Particle;
pub use wrach_bevy::WrachConfig;

/// Main struct for Wrach physics simulations
#[allow(clippy::exhaustive_structs)]
pub struct WrachAPI {
    /// An instance of a Bevy app, already setup for Wrach
    pub app: App,
    /// All the positions of the particles
    pub positions: Vec<(f32, f32)>,
    /// All the velocities of the particles
    pub velocities: Vec<(f32, f32)>,
}

impl WrachAPI {
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

#[allow(clippy::indexing_slicing)]
#[allow(clippy::default_numeric_fallback)]
#[cfg(test)]
mod test {
    use bevy::math::Vec2;

    use super::*;

    #[test]
    fn test_api_returns_data() {
        let mut wrach = WrachAPI::new(WrachConfig {
            dimensions: (10, 10),
            cell_size: 3,
            ..Default::default()
        });

        let mut particles: Vec<Particle> = Vec::new();
        for _ in 0..3 {
            particles.push(Particle {
                position: Vec2::new(5.0, 5.0),
                velocity: Vec2::new(0.5, 0.5),
            });
        }
        wrach.add_particles(particles);

        for _ in 0..5 {
            wrach.tick();
        }

        assert_eq!(wrach.positions.len(), 164);
        assert_ne!(wrach.positions[0], (0.0, 0.0));
        assert_eq!(wrach.velocities.len(), 164);
        assert_ne!(wrach.velocities[0], (0.0, 0.0));
    }
}
