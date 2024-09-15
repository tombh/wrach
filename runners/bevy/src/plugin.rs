//! Setup the Wrach Bevy plugin
use bevy::{asset::embedded_asset, prelude::*};
use bevy_easy_compute::prelude::*;

use crate::{compute::PhysicsComputeWorker, WrachConfig, WrachState};

/// The Wrach Bevy Plugin
#[allow(clippy::exhaustive_structs)]
pub struct WrachPlugin {
    /// All the user-defineable config for Wrach
    pub config: WrachConfig,
}

impl Default for WrachPlugin {
    #[inline]
    fn default() -> Self {
        Self {
            config: WrachConfig::default(),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Plugin for WrachPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../../../assets/shaders/wrach_physics.spv");

        app.insert_resource(WrachState::new(self.config))
            .add_plugins(AppComputePlugin)
            .add_plugins(AppComputeWorkerPlugin::<PhysicsComputeWorker>::default())
            .add_systems(PreUpdate, maybe_upload_to_gpu)
            .add_systems(Update, tick);
    }
}

/// Upload data to the GPU.
/// It's not uploaded immediately but queued to be uploaded with the next wgpu `.submit()`
//
// TODO: Is writing a small amount of data actually a performance improvement, or does it just
// end up writing the whole buffer anywway?
fn maybe_upload_to_gpu(
    mut compute_worker: ResMut<AppComputeWorker<PhysicsComputeWorker>>,
    mut wrach_state: ResMut<WrachState>,
) {
    if wrach_state.gpu_uploads.is_empty() {
        return;
    }

    for uploads in &wrach_state.gpu_uploads {
        if !uploads.positions.is_empty() {
            compute_worker.write_slice(
                PhysicsComputeWorker::POSITIONS_BUFFER_IN,
                &uploads.positions,
            );
        }

        if !uploads.velocities.is_empty() {
            compute_worker.write_slice(
                PhysicsComputeWorker::VELOCITIES_BUFFER_IN,
                &uploads.velocities,
            );
        }
    }

    wrach_state.gpu_uploads = Vec::default();
}

/// What to do for every frame/tick of the simulation
//
// Is there a way for a bevy system to receive a reference to a Resource
#[allow(clippy::needless_pass_by_value)]
fn tick(
    compute_worker: Res<AppComputeWorker<PhysicsComputeWorker>>,
    mut wrach_state: ResMut<WrachState>,
) {
    if !compute_worker.ready() {
        return;
    };

    wrach_state.positions = compute_worker.read_vec(PhysicsComputeWorker::POSITIONS_BUFFER_OUT);
    wrach_state.velocities = compute_worker.read_vec(PhysicsComputeWorker::VELOCITIES_BUFFER_OUT);
}
