use crate::wrach_glam::glam;
use crate::wrach_glam::glam::Vec2;
use cfg_if;

use crate::neighbours;
use crate::particle;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const NUM_PARTICLES: usize = 15000;
        pub const MAP_WIDTH: u32 = 200;
        pub const MAP_HEIGHT: u32 = 200;
    } else {
        pub const NUM_PARTICLES: usize = 4;
        pub const MAP_WIDTH: u32 = 3;
        pub const MAP_HEIGHT: u32 = 3;
    }
}

pub const MAP_SIZE: usize = neighbours::GRID_WIDTH as usize * neighbours::GRID_HEIGHT as usize;

pub const G: Vec2 = glam::const_vec2!([0.0, -1.0]);

pub type PixelMapBasic = [particle::ParticleID; MAP_SIZE];
