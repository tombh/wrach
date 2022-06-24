use crate::event_loop::event_loop::{EventLoop, Renderer};

mod event_loop;
mod gpu_manager;
mod pipeline;

struct SquareVertex;

impl SquareVertex {
    fn init_render_pass<'a>(
        command_encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        // create render pass descriptor and its color attachments
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
    fn render_pass<'a, T: Renderer>(
        &self,
        event_loop: &mut EventLoop<'_, T>,
        command_encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) {
        let index = event_loop.bind_group_index_toggled();
        let particle_buffer = event_loop.manager.pipeline.particle_buffers[index].slice(..);

        command_encoder.push_debug_group("render pixels");
        {
            let mut rpass = Self::init_render_pass(command_encoder, view);
            rpass.set_pipeline(&event_loop.manager.pipeline.render_pipeline);
            rpass.set_vertex_buffer(0, particle_buffer);
            // Verticles that draw the little square "pixel"
            rpass.set_vertex_buffer(1, event_loop.manager.pipeline.vertices_buffer.slice(..));
            rpass.draw(0..6, 0..shaders::world::NUM_PARTICLES as u32);
        }
        command_encoder.pop_debug_group();
    }
}

fn main() {
    EventLoop::run(&SquareVertex {})
}
