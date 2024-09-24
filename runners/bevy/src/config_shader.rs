//! All the config needed for the physics shader to work

// The `ShaderType` derive seems to create code where the final field is duplicated
#![allow(clippy::shadow_reuse)]

use bevy::prelude::Vec2;
use bevy::render::render_resource::ShaderType;
use bevy::{math::UVec2, prelude::Resource};
use bytemuck::{Pod, Zeroable};

/// Config for the shader about the simulation world
#[derive(ShaderType, Resource, Pod, Zeroable, Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct ShaderWorldSettings {
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
