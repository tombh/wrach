pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

impl<'a> Spawner<'a> {
    pub fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    pub fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

impl Default for Spawner<'_> {
    fn default() -> Self {
        Spawner::new()
    }
}
