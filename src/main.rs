mod event_loop;
mod gpu_manager;
mod pipeline;

fn main() {
    let (setup, event_loop) = pollster::block_on(gpu_manager::GPUManager::setup());
    gpu_manager::GPUManager::start(setup, event_loop);
}
