//! All the state for the simulation, both the physics itself and state for managing the simulation

use bevy::{
    asset::Handle,
    math::{Vec2, Vec4},
    prelude::{Resource, Shader},
};

use crate::{
    config_shader::ShaderWorldSettings, particle_store::ParticleStore, spatial_bin::PackedData,
    WrachConfig,
};

/// All simulation state, exported for end users
#[derive(Resource)]
#[allow(clippy::exhaustive_structs)]
pub struct WrachState {
    /// User-defined config
    pub config: WrachConfig,
    /// Settings for the shaders
    pub shader_settings: ShaderWorldSettings,
    /// Store for all particles
    pub particle_store: ParticleStore,
    /// The particle positions
    pub packed_data: PackedData,
    /// Data to send to the GPU, typically for CPU-side influence over the simulation
    pub gpu_uploads: Vec<GPUUpload>,

    /// This is a bit of hack. The types shader is shared by various WGSL shaders, but its asset
    /// handle is not actually consumed by any of them. So we consume it here so that the asset
    /// server doesn't garbage collect it.
    pub types_shader_handle: Option<Handle<Shader>>,
}

/// Wrach's representation of a particle. Probably will only ever be used for inserting.
#[derive(Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct Particle {
    /// Position of particle in units of `WrachConfig::dimensions`
    pub position: Position,
    /// Velocity of particle in x/y components
    pub velocity: Velocity,
}

/// Wrach's type for particle position
pub type Position = Vec2;
/// Wrach's type for particle velocity
pub type Velocity = Vec2;

/// The various kinds of data that get uplaoded to the GPU
pub enum GPUUpload {
    /// The main particle data
    PackedData(PackedData),
    /// Various settings like viewport dimensions, particle count etc
    Settings(ShaderWorldSettings),
}

impl WrachState {
    /// Instantiate
    #[inline]
    #[must_use]
    pub fn new(config: WrachConfig) -> Self {
        let viewport = Vec4::new(
            0.0,
            0.0,
            config.dimensions.0.into(),
            config.dimensions.1.into(),
        );
        Self {
            config,
            shader_settings: ShaderWorldSettings::default(),
            particle_store: ParticleStore::new(config.cell_size, viewport),
            packed_data: PackedData::default(),
            gpu_uploads: Vec::new(),
            types_shader_handle: None,
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
        for particle in particles {
            self.particle_store.add_particle(particle);
        }

        let upload = GPUUpload::PackedData(self.particle_store.create_packed_data());
        self.gpu_upload(upload);

        self.shader_settings.particles_in_frame_count =
            self.particle_store.particles_in_frame_count;
        self.gpu_upload(GPUUpload::Settings(self.shader_settings));
    }
}
