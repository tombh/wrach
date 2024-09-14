//! Rust interface to Wrach simulations

use bevy::prelude::PluginGroup;
use bevy::{app::App, winit::WinitPlugin, DefaultPlugins};
use wrach_bevy::{Particle, WrachPlugin, WrachState};

/// Main struct for Wrach physics simulations
#[non_exhaustive]
pub struct Wrach {
    /// An instance of a Bevy app, already setup for Wrach
    pub app: App,
    /// All the positions of the particles
    pub positions: Vec<(f32, f32)>,
    /// All the velocities of the particles
    pub velocities: Vec<(f32, f32)>,
}

impl Wrach {
    /// Instantiate
    #[must_use]
    #[inline]
    pub fn new(max_particles: u32) -> Self {
        let mut wrach = Self {
            app: App::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
        };
        wrach
            .app
            .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
            .add_plugins(WrachPlugin { max_particles });
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
        self.positions = self
            .app
            .world()
            .resource::<WrachState>()
            .positions
            .iter()
            .map(|particle| (particle.x, particle.y))
            .collect();

        self.velocities = self
            .app
            .world()
            .resource::<WrachState>()
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
}

#[allow(clippy::indexing_slicing)]
#[allow(clippy::default_numeric_fallback)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_api_returns_data() {
        let mut wrach = Wrach::new(3);

        let mut particles: Vec<Particle> = Vec::new();
        for _ in 0..3 {
            particles.push(Particle {
                position: (0.5, 0.5),
                velocity: (0.5, 0.5),
            });
        }
        wrach.add_particles(particles);

        for _ in 0..3 {
            wrach.tick();
        }

        assert_eq!(wrach.positions.len(), 3);
        assert_ne!(wrach.positions[0], (0.0, 0.0));
        assert_eq!(wrach.velocities.len(), 3);
        assert_ne!(wrach.velocities[0], (0.0, 0.0));
    }
}
