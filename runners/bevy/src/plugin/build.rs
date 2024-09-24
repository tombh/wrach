//! Setup the Wrach Bevy plugin

use bevy::{asset::embedded_asset, prelude::*};
use bevy_easy_compute::prelude::*;

use crate::{
    compute::PhysicsComputeWorker, plugin::bind_groups::get_buffers_for_renderer,
    spatial_bin::PackedData, state::GPUUpload, WrachConfig, WrachState,
};

use super::bind_groups::ParticleBindGroupLayout;

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
        embed_shaders(app);

        let mut state = WrachState::new(self.config);

        let types_shader_handle: Option<Handle<Shader>> = Some(
            app.world()
                .resource::<AssetServer>()
                .load("embedded://wrach_bevy/plugin/../../../../assets/shaders/types.wgsl"),
        );
        state.types_shader_handle = types_shader_handle;

        app.insert_resource(state)
            .add_plugins(AppComputePlugin)
            .add_plugins(AppComputeWorkerPlugin::<PhysicsComputeWorker>::default())
            .add_systems(Startup, get_buffers_for_renderer)
            .add_systems(PreUpdate, maybe_upload_to_gpu)
            .add_systems(Update, tick);
    }

    #[inline]
    fn finish(&self, app: &mut App) {
        app.init_resource::<ParticleBindGroupLayout>();
    }
}

/// Embed the shaders into the binary itself
fn embed_shaders(app: &mut App) {
    embedded_asset!(app, "../../../../assets/shaders/wrach_physics.spv");
    embedded_asset!(app, "../../../../assets/shaders/types.wgsl");
    embedded_asset!(app, "../../../../assets/shaders/particles_per_cell.wgsl");
    embedded_asset!(app, "../../../../assets/shaders/prefix_sum.wgsl");
    embedded_asset!(
        app,
        "../../../../assets/shaders/pack_new_particle_data.wgsl"
    );
    embedded_asset!(app, "../../../../assets/shaders/render.wgsl");
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

    for upload in &wrach_state.gpu_uploads {
        match *upload {
            // I don't understand the `ref` keyword. `&` gives a "mismatched types" error.
            #[allow(clippy::ref_patterns)]
            GPUUpload::PackedData(ref data) => {
                debug!("Uploading packed data");

                if !data.indices.is_empty() {
                    compute_worker.write_slice(PhysicsComputeWorker::INDICES_BUFFER, &data.indices);
                }

                if !data.positions.is_empty() {
                    compute_worker
                        .write_slice(PhysicsComputeWorker::POSITIONS_BUFFER_IN, &data.positions);
                }

                if !data.velocities.is_empty() {
                    compute_worker
                        .write_slice(PhysicsComputeWorker::VELOCITIES_BUFFER_IN, &data.velocities);
                }
            }

            GPUUpload::Settings(settings) => {
                debug!("Uploading settings: {:?}", settings);
                compute_worker.write(PhysicsComputeWorker::WORLD_SETTINGS_UNIFORM, &settings);
            }
        }
    }

    wrach_state.gpu_uploads = Vec::new();
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

    let update = PackedData {
        indices: compute_worker.read_vec(PhysicsComputeWorker::INDICES_BUFFER),
        positions: compute_worker.read_vec(PhysicsComputeWorker::POSITIONS_BUFFER_IN),
        velocities: compute_worker.read_vec(PhysicsComputeWorker::VELOCITIES_BUFFER_IN),
    };

    wrach_state.packed_data.indices.clone_from(&update.indices);
    wrach_state
        .packed_data
        .positions
        .clone_from(&update.positions);
    wrach_state
        .packed_data
        .velocities
        .clone_from(&update.velocities);
}
