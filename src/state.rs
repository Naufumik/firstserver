use std::sync::atomic::AtomicI32;
use parking_lot::RwLock;
use crate::models::task::Task;

pub struct AppState {
    pub tasks: RwLock<Vec<Task>>,
    pub next_id: AtomicI32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(Vec::new()),
            next_id: AtomicI32::new(1),
        }
    }
}