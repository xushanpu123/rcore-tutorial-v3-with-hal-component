//!Implementation of [`Processor`] and Intersection of control flow
use super::{fetch_task, TaskStatus, TaskControlBlock};
use crate::sync::UPSafeCell;
use polyhal::pagetable::PageTable;
// use crate::trap::TrapContext;
use polyhal::{kernel_page_table, KContext, context_switch_pt};

use alloc::sync::Arc;
use lazy_static::*;
///Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: KContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: KContext::blank(),
        }
    }
    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut KContext {
        &mut self.idle_task_cx as *mut _
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}
///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const KContext;
            task_inner.task_status = TaskStatus::Running;
            let token = task_inner.memory_set.token();
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe { context_switch_pt(idle_task_cx_ptr, next_task_cx_ptr, token) }
        }
    }
}
///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}
///Get running task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}
///Get token of the address space of current task
pub fn current_user_token() -> PageTable {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}
///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut KContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        context_switch_pt(switched_task_cx_ptr, idle_task_cx_ptr, kernel_page_table());
    }
}
