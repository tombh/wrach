//! Bind groups for the renderer. The compute shader manages these buffers in the main world (as
//! the `bevy_easy_compute` plugin currently dictates), so they need to be extracted to the render
//! world.
//
// TODO: Show that these can be used downstream by a custom renderer.

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, ShaderStages,
        },
        renderer::RenderDevice,
    },
};
use bevy_easy_compute::prelude::*;

use crate::{compute::PhysicsComputeWorker, config_shader::ShaderWorldSettings};

/// The bind group layout for the minimal data needed to render particle
#[derive(Resource, ExtractResource, Clone)]
pub struct ParticleBindGroupLayout {
    /// The bind group layout itself
    pub bind_group_layout: BindGroupLayout,
}

impl FromWorld for ParticleBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = render_device.create_bind_group_layout(
            "ParticlesLayout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ShaderWorldSettings>(false),
                    storage_buffer::<Vec<Vec2>>(false),
                ),
            ),
        );

        Self { bind_group_layout }
    }
}

/// The bind group data for rendering particles as pixels
#[derive(Resource, ExtractResource, Clone)]
pub struct ParticleBindGroup {
    /// The bind group itself
    pub bind_group: BindGroup,
}

pub fn get_buffers_for_renderer(world: &mut World) {
    let render_device = world.resource::<RenderDevice>();
    let bind_group_layout = world.resource::<ParticleBindGroupLayout>();
    let compute_worker = world.resource::<AppComputeWorker<PhysicsComputeWorker>>();

    let bind_group = render_device.create_bind_group(
        None,
        &bind_group_layout.bind_group_layout,
        &BindGroupEntries::sequential((
            #[allow(clippy::expect_used)]
            compute_worker
                .buffers
                .get(PhysicsComputeWorker::WORLD_SETTINGS_UNIFORM)
                .expect("Couldn't get world settings buffer")
                .as_entire_binding(),
            #[allow(clippy::expect_used)]
            compute_worker
                .buffers
                .get(PhysicsComputeWorker::POSITIONS_BUFFER_IN)
                .expect("Couldn't get particle positions buffer")
                .as_entire_binding(),
        )),
    );

    let bindings = ParticleBindGroup { bind_group };
    world.insert_resource(bindings);
}
