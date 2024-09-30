//! Bevy plugin for Wrach 2D pixel physics

// Apparently `pub use` is bad?
// https://rust-lang.github.io/rust-clippy/master/index.html#/pub_use
#![allow(clippy::pub_use)]

/// Tests
#[cfg(test)]
mod tests {
    pub mod utils;
}

/// All GPU-compute related code
mod compute {
    pub use builder::PhysicsComputeWorker;
    pub mod buffers;
    mod builder;

    #[path = "01_particles_cell_count.rs"]
    mod particles_cell_count;

    #[path = "02_prefix_sum.rs"]
    mod prefix_sum;

    #[path = "03_pack_particle_data.rs"]
    mod pack_particle_data;

    #[path = "04_integration.rs"]
    mod integration;
}
mod config_app;
mod config_shader;
mod particle_store;
/// The Bevy Wrach plugin
mod plugin {
    pub mod bind_groups;
    pub mod build;
}
/// Rendering code
mod render {
    pub mod draw_plugin;
    mod graph_node;
    mod pipeline;
}
mod spatial_bin;
mod state;

pub use crate::config_app::WrachConfig;
pub use crate::plugin::build::WrachPlugin;
pub use crate::render::draw_plugin::DrawPlugin;
pub use crate::spatial_bin::PackedData;
pub use crate::state::Particle;
pub use crate::state::WrachState;
