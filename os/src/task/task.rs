//! Types related to task management

use polyhal::kcontext::KContext;
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: KContext,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
