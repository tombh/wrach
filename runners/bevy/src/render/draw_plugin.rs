//! A simple renderer that just draws each particle as a small pixel

use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        render_graph::{RenderGraphApp as _, ViewNodeRunner},
        MainWorld, RenderApp,
    },
};

use crate::{
    config_shader::ShaderWorldSettings,
    plugin::bind_groups::{ParticleBindGroup, ParticleBindGroupLayout},
    render::graph_node::{DrawParticleLabel, DrawParticleNode},
    WrachState,
};

use super::pipeline::DrawParticlePipeline;

/// An optional plugin to draw particles as simple pixels
#[derive(Default)]
#[non_exhaustive]
pub struct DrawPlugin;

#[expect(clippy::missing_trait_methods, reason = "We just don't need to others")]
impl Plugin for DrawPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                ExtractSchedule,
                (setup.run_if(check_is_setup), sync_world_settings),
            )
            .add_render_graph_node::<ViewNodeRunner<DrawParticleNode>>(Core2d, DrawParticleLabel)
            .add_render_graph_edge(Core2d, Node2d::Tonemapping, DrawParticleLabel);
    }
}

/// Startup system for [`DrawPlugin`]
fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Simple system to check that our custom setup has happened.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy's system function signature can't be changed"
)]
const fn check_is_setup(maybe: Option<Res<ParticleBindGroupLayout>>) -> bool {
    maybe.is_none()
}

/// We don't use the traditional `Setup` schedule because bindgroups aren't ready from the
/// compute plugin and the `PipelineCache` isn't ready for [`DrawParticlePipeline`].
/// I'd like to know if there's a better way of doing this?
#[expect(
    clippy::expect_used,
    reason = "`expect`s until there's a way to use `?` in systems"
)]
fn setup(mut commands: Commands, mut world: ResMut<MainWorld>) {
    let particle_bind_group_layout = world
        .remove_resource::<ParticleBindGroupLayout>()
        .expect("Couldn't remove `ParticleBindGroupLayout` from main world");
    let particle_bind_group = world
        .remove_resource::<ParticleBindGroup>()
        .expect("Couldn't remove `ParticleBindGroup` from main world");
    commands.insert_resource(particle_bind_group_layout);
    commands.insert_resource(particle_bind_group);

    commands.init_resource::<DrawParticlePipeline>();
}

/// Synchronise the [`ShaderWorldSettings`] from the main world so they can be used in the shader.
/// Sync happens for every frame.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy's magic system function signature can't be changed"
)]
fn sync_world_settings(mut commands: Commands, world: ResMut<MainWorld>) {
    let state = world.resource::<WrachState>();
    commands.insert_resource::<ShaderWorldSettings>(state.shader_settings);
}
