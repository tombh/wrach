use crevice::std140::AsStd140;
use std::time::{Duration, Instant};
use winit::event::{self, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::platform::run_return::EventLoopExtRunReturn;

use super::spawner;
use crate::gpu_manager;

pub struct EventLoop<'a> {
    manager: gpu_manager::GPUManager,
    is_paused: bool,
    is_exit: bool,
    spawner: spawner::Spawner<'a>,
    last_frame_inst: Instant,
    last_update_inst: Instant,
    frame_count: u64,
    accum_time: f32,
}

impl EventLoop<'_> {
    pub fn run() {
        let (manager, event_loop) = pollster::block_on(gpu_manager::GPUManager::setup());
        let instance = Self {
            manager,
            is_paused: false,
            is_exit: false,
            spawner: spawner::Spawner::new(),
            last_update_inst: Instant::now(),
            last_frame_inst: Instant::now(),
            frame_count: 0,
            accum_time: 0.0,
        };

        instance.enter(event_loop);
    }

    pub fn enter(mut self, mut event_loop: winit::event_loop::EventLoop<()>) {
        log::info!("Entering render loop...");
        event_loop.run_return(move |event, _, control_flow| {
            // Only captured so they're droppped
            let _ = (&self.manager.instance, &self.manager.adapter);

            // TODO How to set the lifetimes to make `control_flow` a part of `self`?
            if self.is_exit {
                *control_flow = ControlFlow::Exit;
            } else {
                *control_flow = ControlFlow::Poll;
            }
            self.handle_event(event, control_flow);
        });
    }

    fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        match event {
            event::Event::RedrawEventsCleared => self.redraw_events_cleared(control_flow),
            event::Event::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                log::info!("Resizing to {:?}", size);
                self.manager.config.width = size.width.max(1);
                self.manager.config.height = size.height.max(1);
                self.manager.pipeline.resize(
                    &self.manager.config,
                    &self.manager.device,
                    &self.manager.queue,
                );
                self.manager
                    .surface
                    .configure(&self.manager.device, &self.manager.config);
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    self.is_exit = true;
                    return;
                }
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => match (virtual_code, state) {
                    (event::VirtualKeyCode::R, event::ElementState::Pressed) => {
                        println!("R");
                        // println!("{:#?}", instance.generate_report());
                    }
                    (event::VirtualKeyCode::Space, event::ElementState::Pressed) => {
                        self.toggle_pause();
                    }
                    (event::VirtualKeyCode::Escape, event::ElementState::Pressed) => {
                        self.is_exit = true;
                    }
                    _ => (),
                },

                _ => {
                    self.manager.pipeline.update(event);
                }
            },
            event::Event::RedrawRequested(_) => self.redraw_requestsed(),
            _ => {}
        }
    }

    fn redraw_events_cleared(&mut self, _control_flow: &mut winit::event_loop::ControlFlow) {
        // Clamp to some max framerate to avoid busy-looping too much
        // (we might be in wgpu::PresentMode::Mailbox, thus discarding superfluous frames)
        //
        // winit has window.current_monitor().video_modes() but that is a list of all full screen video modes.
        // So without extra dependencies it's a bit tricky to get the max refresh rate we can run the window on.
        // Therefore we just go with 60fps - sorry 120hz+ folks!
        let target_frametime = Duration::from_secs_f64(1.0 / 300.0);
        let time_since_last_frame = self.last_update_inst.elapsed();
        if time_since_last_frame >= target_frametime {
            // self.manager.window.request_redraw();
            self.last_update_inst = Instant::now();
        } else {
            // *control_flow = winit::event_loop::ControlFlow::WaitUntil(
            //     Instant::now() + target_frametime - time_since_last_frame,
            // );
        }
        self.manager.window.request_redraw();
        self.spawner.run_until_stalled();
    }

    fn redraw_requestsed(&mut self) {
        self.accum_time += self.last_frame_inst.elapsed().as_secs_f32();
        self.last_frame_inst = Instant::now();
        if self.frame_count > 100 {
            println!(
                "Avg frame time {}ms",
                self.accum_time * 1000.0 / self.frame_count as f32
            );
            self.accum_time = 0.0;
            self.frame_count = 0;
        }

        let frame = match self.manager.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.manager
                    .surface
                    .configure(&self.manager.device, &self.manager.config);
                self.manager
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render(&view);
        frame.present();
    }

    fn init_command_encoder(manager: &gpu_manager::GPUManager) -> wgpu::CommandEncoder {
        manager
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

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

    fn bind_group_index_toggled(&mut self) -> usize {
        if self.manager.pipeline.bind_group == 0 {
            self.manager.pipeline.bind_group = 1;
        } else {
            self.manager.pipeline.bind_group = 0;
        }
        self.manager.pipeline.bind_group
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Called for every frame
    pub fn render(&mut self, view: &wgpu::TextureView) {
        if self.is_paused {
            return;
        }

        let mut command_encoder = Self::init_command_encoder(&self.manager);
        self.compute_pass(&mut command_encoder);
        self.render_pass(&mut command_encoder, view);
        self.manager.queue.submit(Some(command_encoder.finish()));
        self.frame_count += 1;
    }

    fn compute_pass<'a>(&mut self, command_encoder: &'a mut wgpu::CommandEncoder) {
        command_encoder.clear_buffer(&self.manager.pipeline.grid_buffer, 0, None);
        self.pre_compute_pass(command_encoder);
        command_encoder.push_debug_group("compute");
        {
            for _ in 0..shaders::particle::DEFAULT_NUM_SOLVER_SUBSTEPS {
                self.compute_pass_stage(command_encoder, 0);
                self.compute_pass_stage(command_encoder, 1);
                self.compute_pass_stage(command_encoder, 2);
            }
        }
        command_encoder.pop_debug_group();
    }

    fn pre_compute_pass<'a>(&mut self, command_encoder: &'a mut wgpu::CommandEncoder) {
        let mut cpass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        let index = self.bind_group_index_toggled();
        let bind_groups = &self.manager.pipeline.particle_bind_groups[index];
        cpass.set_bind_group(0, bind_groups, &[]);
        cpass.set_pipeline(&self.manager.pipeline.pre_compute_pipeline);
        cpass.dispatch(self.manager.pipeline.work_group_count, 1, 1);
    }

    fn compute_pass_stage<'a>(
        &mut self,
        command_encoder: &'a mut wgpu::CommandEncoder,
        stage: u32,
    ) {
        let mut cpass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        let index = self.bind_group_index_toggled();
        let bind_groups = &self.manager.pipeline.particle_bind_groups[index];
        cpass.set_bind_group(0, bind_groups, &[]);
        cpass.set_pipeline(&self.manager.pipeline.compute_pipeline);

        let ps = shaders::compute::Params { stage };
        cpass.set_push_constants(0, bytemuck::bytes_of(&ps.as_std140()));
        cpass.dispatch(self.manager.pipeline.work_group_count, 1, 1);
    }

    fn render_pass<'a>(
        &mut self,
        command_encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) {
        let index = self.bind_group_index_toggled();
        let particle_buffer = self.manager.pipeline.particle_buffers[index].slice(..);
        command_encoder.push_debug_group("render pixels");
        {
            let mut rpass = Self::init_render_pass(command_encoder, view);
            rpass.set_pipeline(&self.manager.pipeline.render_pipeline);
            rpass.set_vertex_buffer(0, particle_buffer);
            // Verticles that draw the littel square  "pixel"
            rpass.set_vertex_buffer(1, self.manager.pipeline.vertices_buffer.slice(..));
            rpass.draw(0..6, 0..shaders::world::NUM_PARTICLES as u32);
        }
        command_encoder.pop_debug_group();
    }
}
