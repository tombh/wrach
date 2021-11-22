use crate::wrach_glam::glam;
use crate::wrach_glam::glam::Vec2;
use cfg_if;

pub const WINDOW_ZOOM: u32 = 5;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const NUM_PARTICLES: usize = 1000;
        pub const MAP_WIDTH: u32 = 100;
        pub const MAP_HEIGHT: u32 = 100;
    } else {
        pub const NUM_PARTICLES: usize = 5;
        pub const MAP_WIDTH: u32 = 3;
        pub const MAP_HEIGHT: u32 = 3;
    }
}

pub const G: Vec2 = glam::const_vec2!([0.0, -10.0]);
