use wgpu::util::DeviceExt;
use wrach_physics_shaders as physics;
use wrach_wgpu::event_looper::event_loop::{EventLoop, Renderer};
use wrach_wgpu::pipeliner::builder::Builder;
use wrach_wgpu::{bytemuck, gpu_manager, wgpu};

struct SquareVertex {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
}

impl SquareVertex {
    fn init_render_pipeline(
        shader_module: &wgpu::ShaderModule,
        manager: &gpu_manager::GPUManager,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            manager
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let vec2_array_stride = std::mem::size_of::<physics::wrach_glam::glam::Vec2>() as u64;

        let fragment = wgpu::FragmentState {
            module: shader_module,
            entry_point: "main_fs",
            targets: &[manager.config.format.into()],
        };

        let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: vec2_array_stride,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                    },
                ],
            },
            fragment: Some(fragment),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        };

        manager.device.create_render_pipeline(&pipeline_descriptor)
    }

    fn init_vertices_buffer(manager: &gpu_manager::GPUManager) -> wgpu::Buffer {
        // A square made of 1 triangles
        #[rustfmt::skip]
        let vertex_buffer_data: Vec<f32> = [
            // First triangle ----------------------
            -1, -1, -1, 1, 1, 1,
            // Second triangle ----------------------
            -1, -1, 1, 1, 1, -1,
        ]
        .iter()
        .map(|x| 0.5 * physics::particle::PIXEL_SIZE * (*x as f32))
        .collect();
        let mut square = [0.0; 12];
        (0..12).for_each(|i| {
            square[i] = vertex_buffer_data[i];
        });
        manager
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::bytes_of(&square),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    fn init_render_pass<'a>(
        command_encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let color_attachments = [wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        }];

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        command_encoder.begin_render_pass(&render_pass_descriptor)
    }
}

impl Renderer for SquareVertex {
    fn new(manager: &gpu_manager::GPUManager) -> Self {
        let shader_module = Builder::shader(&manager.device, "shaders/renderer");
        Self {
            pipeline: Self::init_render_pipeline(&shader_module, manager),
            vertices: Self::init_vertices_buffer(manager),
        }
    }

    fn render_pass<'a, T: Renderer>(
        &self,
        event_loop: &mut EventLoop<'_, T>,
        command_encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) {
        let index = event_loop.bind_group_index_toggled();
        let positions_buffer = event_loop.manager.pipeline.position_buffers[index].slice(..);

        command_encoder.push_debug_group("render pixels");
        {
            let mut rpass = Self::init_render_pass(command_encoder, view);
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, positions_buffer);
            rpass.set_vertex_buffer(1, self.vertices.slice(..));
            rpass.draw(0..6, 0..physics::world::NUM_PARTICLES as u32);
        }
        command_encoder.pop_debug_group();
    }
}

fn main() {
    EventLoop::<SquareVertex>::start()
}
