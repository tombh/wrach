//! Rust interface to Wrach simulations

use bevy::prelude::PluginGroup;
use bevy::{app::App, winit::WinitPlugin, DefaultPlugins};
use wrach_bevy::{WrachPlugin, WrachState};

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
    pub fn new(number_of_particles: i32) -> Self {
        let mut wrach = Self {
            app: App::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
        };
        wrach
            .app
            .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
            .add_plugins(WrachPlugin {
                size: number_of_particles,
            });
        wrach.app.finish();
        wrach.app.cleanup();
        wrach
    }

    /// Run a single tick/frame of the simulation
    #[inline]
    pub fn tick(&mut self) {
        self.app.update();
        self.update_data();
    }

    /// Get data from the simulation
    // TODO: Check performance of this. Are we using the data directly? There's no copying going
    // on?
    #[inline]
    pub fn update_data(&mut self) {
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
}

#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_api_returns_data() {
        let mut wrach = Wrach::new(3);
        wrach.tick();

        assert_eq!(wrach.positions.len(), 3);
        assert_ne!(wrach.positions[0], (0.0, 0.0));
        assert_eq!(wrach.velocities.len(), 3);
        assert_ne!(wrach.velocities[0], (0.0, 0.0));
    }
}
