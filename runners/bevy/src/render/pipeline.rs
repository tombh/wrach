//! The render pipeline for drawing particles as simple pixels

use bevy::{
    asset::DirectAssetAccessExt,
    prelude::{FromWorld, Resource, World},
    render::{
        render_resource::{
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            PipelineCache, PrimitiveState, TextureFormat, VertexState,
        },
        texture::BevyDefault,
    },
};

use crate::plugin::bind_groups::ParticleBindGroupLayout;

/// The render pipeline for drawing particles as simple pixels
#[derive(Resource)]
pub struct DrawParticlePipeline {
    /// Cached render pipeline
    pub pipeline: CachedRenderPipelineId,
}

impl FromWorld for DrawParticlePipeline {
    fn from_world(world: &mut World) -> Self {
        let bindings = world.resource::<ParticleBindGroupLayout>();
        let shader =
            world.load_asset("embedded://wrach_bevy/plugin/../../../../assets/shaders/render.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_render_pipeline(
            bevy::render::render_resource::RenderPipelineDescriptor {
                label: None,
                layout: [bindings.bind_group_layout.clone()].to_vec(),
                push_constant_ranges: Vec::new(),
                vertex: VertexState {
                    shader: shader.clone(),
                    entry_point: "vertex".into(),
                    shader_defs: vec![],
                    buffers: vec![],
                },
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            },
        );

        Self { pipeline }
    }
}
