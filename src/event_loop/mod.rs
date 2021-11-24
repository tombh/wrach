pub mod event_loop;
pub mod spawner;

pub fn run() {
    crate::event_loop::event_loop::EventLoop::run();
}
