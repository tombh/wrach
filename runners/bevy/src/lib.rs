//! Bevy plugin for Wrach 2D pixel physics

// Apparently `pub use` is bad?
// https://rust-lang.github.io/rust-clippy/master/index.html#/pub_use
#![allow(clippy::pub_use)]

mod compute;
mod config_app;
mod config_shader;
mod plugin;
mod state;

pub use crate::config_app::WrachConfig;
pub use crate::plugin::WrachPlugin;
pub use crate::state::GPUUpload;
pub use crate::state::Particle;
pub use crate::state::WrachState;
