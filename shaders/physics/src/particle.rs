//! A single particle and everything that can be done to it

use spirv_std::{
    arch::IndexUnchecked,
    glam::{vec4, Vec2},
};

use crate::integrate::WorldConfig;

/// Convenient representation of a particle
pub struct Particle {
    /// Index of the particle
    pub index: usize,
    /// Particle postiion
    pub position: Vec2,
    /// Particle velocity
    pub velocity: Vec2,
}

impl<'particle> Particle {
    /// Instantiate
    pub fn new(
        index: usize,
        positions_input: &'particle [Vec2],
        velocities_input: &'particle [Vec2],
    ) -> Self {
        // SAFETY:
        //   Getting data with bounds checks is obviously undefined behaviour. We rely on the
        //   rest of the pipeline to ensure that indices are always within limits.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            Self {
                index,
                position: *positions_input.index_unchecked(index),
                velocity: *velocities_input.index_unchecked(index),
            }
        }
    }

    /// Enforce particle limits like bouundaries and speed
    pub fn enforce_limits(&mut self, world_config: &WorldConfig) {
        self.enforce_boundaries(world_config);
        self.enforce_velocity();
    }

    /// Enforce particle boundaries
    pub fn enforce_boundaries(&mut self, world_config: &WorldConfig) {
        let viewport = vec4(
            world_config.view_anchor.x,
            world_config.view_anchor.y,
            world_config.view_anchor.x + world_config.dimensions.x,
            world_config.view_anchor.y + world_config.dimensions.y,
        );

        if self.position.x > viewport.z {
            self.position.x = viewport.z;
            self.velocity.x *= -1.0;
        }
        if self.position.x < viewport.x {
            self.position.x = viewport.x;
            self.velocity.x *= -1.0;
        }
        if self.position.y > viewport.w {
            self.position.y = viewport.w;
            self.velocity.y *= -1.0;
        }
        if self.position.y < viewport.y {
            self.position.y = viewport.y;
            self.velocity.y *= -1.0;
        }
    }

    /// Enforce maximum particle velocity
    pub fn enforce_velocity(&mut self) {
        let max = 1.0;
        self.velocity.x = self.velocity.x.clamp(-max, max);
        self.velocity.y = self.velocity.y.clamp(-max, max);
    }

    /// Integration, therefore move the particle by its velocity
    pub fn integrate(&mut self) {
        self.position += self.velocity;
    }

    /// Write particle data back to buffer
    pub fn write(
        &self,
        positions_output: &'particle mut [Vec2],
        velocities_output: &'particle mut [Vec2],
    ) {
        // SAFETY: See same comment for `new()`
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            let pun = positions_output.index_unchecked_mut(self.index);
            *pun = self.position;
            let vun = velocities_output.index_unchecked_mut(self.index);
            *vun = self.velocity;
        }
    }
}
