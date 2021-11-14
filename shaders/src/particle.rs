#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

use crate::wrach_glam::glam::{vec2, Vec2};

#[cfg_attr(
    not(target_arch = "spirv"),
    derive(bytemuck::Pod, bytemuck::Zeroable, Debug)
)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Particle {
    const SIZE: f32 = 0.1;
    const E: f32 = 0.00001;

    pub fn force(&self, other: Particle) -> Vec2 {
        let distance = self.distance(other);
        let ratio: f32;
        if distance >= Particle::SIZE {
            ratio = Particle::SIZE / distance;
        } else {
            ratio = 1.0;
        }
        // ratio = Particle::SIZE / distance;

        let attraction = ratio.powf(12.0);
        let repulsion = ratio.powf(6.0);
        let energy = Particle::E * (attraction - repulsion);

        let mut force = (2.0 * energy).abs().sqrt();
        if energy < 0.0 {
            force = -force;
        }

        let angle = self.angle(other);
        let mut force_x = angle.cos();
        if self.position.x < other.position.x {
            force_x = -force_x;
        }
        let mut force_y = angle.sin();
        if self.position.y < other.position.y {
            force_y = -force_y;
        }
        return vec2(force_x, force_y) * force;
    }

    pub fn distance(&self, other: Particle) -> f32 {
        self.position.distance(other.position)
    }

    pub fn angle(&self, other: Particle) -> f32 {
        let x_distance = self.position.x - other.position.x;
        let y_distance = self.position.y - other.position.y;
        let ratio = y_distance / x_distance;
        let angle = ratio.atan();
        angle
    }

    pub fn bounce_off_walls(&mut self) {
        if self.position.x < -1.0 {
            self.velocity.x *= -1.0;
        }
        if self.position.x > 1.0 {
            self.velocity.x *= -1.0;
        }
        if self.position.y < -1.0 {
            self.velocity.y *= -1.0;
        }
        if self.position.y > 1.0 {
            self.velocity.y *= -1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MIN_FORCE: f32 = 1.0e-7;
    const ZEROISH_FORCE: f32 = 1.0e-13;

    macro_rules! is_zeroish_force {
        ($force:expr) => {
            assert!(
                $force.abs() < ZEROISH_FORCE,
                "force is not zeroish {:?}",
                $force
            );
        };
    }

    macro_rules! is_positive_force {
        ($force:expr) => {
            assert!(
                $force > 0.0 && $force > MIN_FORCE,
                "force is not positive {:?}",
                $force
            );
        };
    }

    macro_rules! is_negative_force {
        ($force:expr) => {
            assert!(
                $force < 0.0 && $force < -MIN_FORCE,
                "force is not negative {:?}",
                $force
            );
        };
    }

    fn top_left() -> Particle {
        Particle {
            position: vec2(0.0, 1.0),
            velocity: vec2(0.0, 0.0),
        }
    }

    fn top_right() -> Particle {
        Particle {
            position: vec2(1.0, 1.0),
            velocity: vec2(0.0, 0.0),
        }
    }

    fn _centre() -> Particle {
        Particle {
            position: vec2(0.5, 0.5),
            velocity: vec2(0.0, 0.0),
        }
    }

    fn bottom_left() -> Particle {
        Particle {
            position: vec2(0.0, 0.0),
            velocity: vec2(0.0, 0.0),
        }
    }
    fn _bottom_right() -> Particle {
        Particle {
            position: vec2(1.0, 0.0),
            velocity: vec2(0.0, 0.0),
        }
    }

    #[test]
    fn it_calculates_the_force_between_particles() {
        let force = top_left().force(top_right());
        is_positive_force!(force.x);
        is_zeroish_force!(force.y);

        let force = top_right().force(top_left());
        is_negative_force!(force.x);
        is_zeroish_force!(force.y);

        let force = top_left().force(bottom_left());
        is_zeroish_force!(force.x);
        is_negative_force!(force.y);

        let force = bottom_left().force(top_right());
        let bltrx = force.x;
        let bltry = force.y;
        is_positive_force!(force.x);
        is_positive_force!(force.y);

        let force = top_right().force(bottom_left());
        is_negative_force!(force.x);
        is_negative_force!(force.y);

        assert_eq!(bltrx, -force.x);
        assert_eq!(bltry, -force.y);
    }
}
