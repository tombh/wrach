//! A custom render node for drawing particles as simple pixels

use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{self, RenderGraphContext, RenderLabel},
        render_resource::{CachedPipelineState, Pipeline, PipelineCache, RenderPassDescriptor},
        renderer::RenderContext,
        view::ViewTarget,
    },
};

use crate::{config_shader::ShaderWorldSettings, plugin::bind_groups::ParticleBindGroup};

use super::pipeline::DrawParticlePipeline;

/// The label for our custom node in the render graph
#[derive(RenderLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrawParticleLabel;

/// Our custom render node in the render graph
#[derive(Default)]
pub struct DrawParticleNode;

#[expect(clippy::missing_trait_methods, reason = "We just don't use 'em")]
impl render_graph::ViewNode for DrawParticleNode {
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_query: QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<DrawParticlePipeline>();
        let settings = world.resource::<ShaderWorldSettings>();
        let bindings = world.resource::<ParticleBindGroup>();

        let color_attachment = view_query.get_color_attachment();

        let mut pass = render_context
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "I, @tombh, copied this from somewhere and am just being lazy in figuring out what the downsides are"
        )]
        if let CachedPipelineState::Ok(pipeline_cached) =
            pipeline_cache.get_render_pipeline_state(pipeline.pipeline)
        {
            #[expect(
                clippy::unreachable,
                reason = "I, @tombh, copied this from somewhere so don't actuall know why it's unreachable"
            )]
            let Pipeline::RenderPipeline(pipeline_ready) = pipeline_cached
            else {
                unreachable!("Cached pipeline isn't ready");
            };

            pass.set_bind_group(0, &bindings.bind_group, &[]);
            pass.set_pipeline(pipeline_ready);
            pass.draw(0..6, 0..settings.particles_in_frame_count);
        }

        Ok(())
    }
}
