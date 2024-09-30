//! Code shared by both the CPU and GPU

#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::as_conversions)]
#![allow(clippy::too_many_arguments)]

#[cfg(not(target_arch = "spirv"))]
use bevy::prelude::{UVec2, Vec2};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{UVec2, Vec2};

/// Config needed by the simulation
#[allow(clippy::exhaustive_structs)]
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
