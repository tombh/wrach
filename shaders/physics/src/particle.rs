//! A single particle and everything that can be done to it

use spirv_std::{
    arch::IndexUnchecked,
    glam::{vec4, Vec2},
};
use wrach_cpu_gpu_shared::WorldSettings;

/// Convenient representation of a particle
#[derive(Default, Copy, Clone)]
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
    pub fn enforce_limits(&mut self, world_config: &WorldSettings) {
        self.enforce_boundaries(world_config);
        self.enforce_velocity();
    }

    /// Enforce particle boundaries
    pub fn enforce_boundaries(&mut self, world_config: &WorldSettings) {
        let viewport = vec4(
            world_config.view_anchor.x,
            world_config.view_anchor.y,
            world_config.view_anchor.x + world_config.view_dimensions.x,
            world_config.view_anchor.y + world_config.view_dimensions.y,
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
    pub fn write(&self, positions_output: &mut [Vec2], velocities_output: &mut [Vec2]) {
        // SAFETY: See same comment for `new()`
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            let position_reference = positions_output.index_unchecked_mut(self.index);
            *position_reference = self.position;

            let velocity_reference = velocities_output.index_unchecked_mut(self.index);
            *velocity_reference = self.velocity;

            // TODO:
            //   Wait for Bevy 0.15 and its support of Naga's atomics, see:
            //   https://github.com/gfx-rs/wgpu/issues/4489
            //   This will allow us to remove a whole GPU pass.
            //
            // let count_reference = indices_counter.index_unchecked_mut(self.index);
            // spirv_std::arch::atomic_i_increment::<
            //     _,
            //     { spirv_std::memory::Scope::Device as u32 },
            //     { spirv_std::memory::Semantics::NONE.bits() },
            // >(count_reference);
        }
    }
}
