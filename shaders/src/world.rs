use crate::wrach_glam::glam;
use crate::wrach_glam::glam::Vec2;
use cfg_if;

use crate::particle;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const NUM_PARTICLES: usize = 150;
        pub const MAP_WIDTH: u32 = 800;
        pub const MAP_HEIGHT: u32 = 800;
    } else {
        pub const NUM_PARTICLES: usize = 4;
        pub const MAP_WIDTH: u32 = 3;
        pub const MAP_HEIGHT: u32 = 3;
    }
}

pub const MAP_SIZE: usize = MAP_WIDTH as usize * MAP_HEIGHT as usize;

pub const G: Vec2 = glam::const_vec2!([0.0, -0.0]);

pub type PixelMapBasic = [particle::ParticleID; MAP_SIZE];
