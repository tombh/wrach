pub mod event_loop;
pub mod spawner;

use crate::gpu_manager;

pub fn run(gpu_manager: gpu_manager::GPUManager, event_loop: winit::event_loop::EventLoop<()>) {
    crate::event_loop::event_loop::EventLoop::run(gpu_manager, event_loop);
}
