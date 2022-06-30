use crate::wrach_glam::glam;
use crate::wrach_glam::glam::Vec2;
use cfg_if;

pub const WINDOW_ZOOM: u32 = 3;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const NUM_PARTICLES: usize = 5000;
        pub const MAP_WIDTH: u32 = 300;
        pub const MAP_HEIGHT: u32 = 300;
    } else {
        pub const NUM_PARTICLES: usize = 4;
        pub const MAP_WIDTH: u32 = 3;
        pub const MAP_HEIGHT: u32 = 3;
    }
}

pub const G: Vec2 = glam::const_vec2!([0.0, -10.0]);
