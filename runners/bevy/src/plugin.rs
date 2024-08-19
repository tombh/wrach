//! Setup the Wrach Bevy plugin
use bevy::{asset::embedded_asset, prelude::*};
use bevy_easy_compute::prelude::*;

use crate::{compute::PhysicsComputeWorker, WrachState};

/// All the config for the Wrach Bevy plugin
#[allow(clippy::exhaustive_structs)]
pub struct WrachPlugin {
    /// Number of particles/pixels to simulate
    pub size: i32,
}

#[allow(clippy::missing_trait_methods)]
impl Plugin for WrachPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../../../assets/shaders/wrach_physics.spv");

        app.insert_resource(WrachState::new(self.size))
            .add_plugins(AppComputePlugin)
            .add_plugins(AppComputeWorkerPlugin::<PhysicsComputeWorker>::default())
            .add_systems(Update, tick);
    }
}

/// What to do for every frame/tick of the simulation
fn tick(
    mut compute_worker: ResMut<AppComputeWorker<PhysicsComputeWorker>>,
    mut wrach_state: ResMut<WrachState>,
) {
    if !compute_worker.ready() {
        return;
    };

    wrach_state.positions = compute_worker.read_vec(PhysicsComputeWorker::POSITIONS_BUFFER_OUT);
    wrach_state.velocities = compute_worker.read_vec(PhysicsComputeWorker::VELOCITIES_BUFFER_OUT);

    // TODO: Is writing a small amoung of data actually a performance improvement, or does it just
    // end up writing the whole buffer anywway?
    if !wrach_state.overwrite.is_empty() {
        compute_worker.write_slice(
            PhysicsComputeWorker::VELOCITIES_BUFFER_IN,
            &wrach_state.overwrite,
        );
        wrach_state.overwrite = Vec::default();
    }
}
