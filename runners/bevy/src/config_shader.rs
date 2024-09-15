//! All the config needed for the physics shader to work

// The `ShaderType` derive seems to create code where the final field is duplicated
#![allow(clippy::shadow_reuse)]

use bevy::prelude::Vec2;
use bevy::render::render_resource::ShaderType;

/// Config for the shader about the simulation world
#[derive(ShaderType)]
pub struct ShaderWorldConfig {
    /// Dimensions of the view onto the simulation
    pub dimensions: Vec2,
    /// Current position of the viewoport. Measured from the bottom-left corner
    pub view_anchor: Vec2,
}
