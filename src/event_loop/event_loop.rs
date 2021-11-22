use std::time::{Duration, Instant};
use winit::event::{self, WindowEvent};
use winit::event_loop::ControlFlow;

use super::spawner;
use crate::gpu_manager;

pub struct EventLoop {
    manager: gpu_manager::GPUManager,
    is_exit: bool,
    spawner: spawner::Spawner<'static>,
    last_frame_inst: Instant,
    last_update_inst: Instant,
    frame_count: u64,
    accum_time: f32,
}

impl EventLoop {
    pub fn run(manager: gpu_manager::GPUManager, event_loop: winit::event_loop::EventLoop<()>) {
        let instance = Self {
            manager,
            is_exit: false,
            spawner: spawner::Spawner::new(),
            last_update_inst: Instant::now(),
            last_frame_inst: Instant::now(),
            frame_count: 0,
            accum_time: 0.0,
        };
        instance.enter(event_loop);
    }

    pub fn enter(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        log::info!("Entering render loop...");
        event_loop.run(move |event, _, control_flow| {
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
                        self.manager.pipeline.toggle_pause();
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

    fn redraw_events_cleared(&mut self, control_flow: &mut winit::event_loop::ControlFlow) {
        // Clamp to some max framerate to avoid busy-looping too much
        // (we might be in wgpu::PresentMode::Mailbox, thus discarding superfluous frames)
        //
        // winit has window.current_monitor().video_modes() but that is a list of all full screen video modes.
        // So without extra dependencies it's a bit tricky to get the max refresh rate we can run the window on.
        // Therefore we just go with 60fps - sorry 120hz+ folks!
        let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
        let time_since_last_frame = self.last_update_inst.elapsed();
        if time_since_last_frame >= target_frametime {
            // window.request_redraw();
            self.last_update_inst = Instant::now();
        } else {
            *control_flow = winit::event_loop::ControlFlow::WaitUntil(
                Instant::now() + target_frametime - time_since_last_frame,
            );
        }
        self.manager.window.request_redraw();
        self.spawner.run_until_stalled();
    }

    fn redraw_requestsed(&mut self) {
        self.accum_time += self.last_frame_inst.elapsed().as_secs_f32();
        self.last_frame_inst = Instant::now();
        self.frame_count += 1;
        if self.frame_count == 100 {
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

        self.manager
            .pipeline
            .render(&view, &self.manager.device, &self.manager.queue);
        frame.present();
    }
}
