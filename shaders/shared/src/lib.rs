//! Code shared by both the CPU and GPU

#![expect(
    stable_features,
    reason = "Remove `feature(lint_reasons)` once `rust-gpu` supports Rust 1.81"
)]
#![feature(lint_reasons)]
#![cfg_attr(target_arch = "spirv", no_std)]

#[cfg(not(target_arch = "spirv"))]
use bevy::prelude::{UVec2, Vec2};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{UVec2, Vec2};

// TODO: Document why we can't share with `ShaderWorldSettings` in `bevy/src/config_shader.rs`.
/// Config needed by the simulation
#[expect(clippy::exhaustive_structs, reason = "")]
#[derive(Default)]
pub struct WorldSettings {
    /// Dimensions of the view onto the simulation
    pub view_dimensions: Vec2,
    /// Current position of the viewoport. Measured from the bottom-left corner
    pub view_anchor: Vec2,
    /// The dimensions of the spatial bin grid, the unit is a cell
    pub grid_dimensions: UVec2,
    /// The size of a spatial bin cell
    pub cell_size: u32,
    /// Total number of particles simulated in this frame. This will normally be much smaller than
    /// the total number of particles that we have a record of.
    pub particles_in_frame_count: u32,
}

/// The size of a single spatial bin cell. The unit is one side of the square.
pub const SPATIAL_BIN_CELL_SIZE: u16 = 2;

/// NB: We add one to cell indexes because our current prefix sum implementation shifts all its items
/// one to the right.
pub const PREFIX_SUM_OFFSET_HACK: u32 = 1;
