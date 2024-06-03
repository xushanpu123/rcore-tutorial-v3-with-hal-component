use super::{fetch_task, TaskStatus};
use super::{ProcessControlBlock, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::task::id;
use alloc::sync::Arc;
use lazy_static::*;
use log::info;
use polyhal::{context_switch, context_switch_pt, kernel_page_table, KContext, TrapFrame};

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: KContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: KContext::blank(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut KContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let page_table = task.page_table_token();
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const KContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            // FIXME: context switch
            // unsafe {
            //     __switch(idle_task_cx_ptr, next_task_cx_ptr);
            // }
            // info!("switch to task: {:#x?}", unsafe { next_task_cx_ptr.as_ref().unwrap() });
            unsafe {
                context_switch_pt(idle_task_cx_ptr, next_task_cx_ptr, page_table);
            }
        } else {
            println!("no tasks available in run_tasks");
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}

pub fn current_trap_cx() -> &'static mut TrapFrame {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

pub fn current_trap_cx_user_va() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .trap_cx_user_va()
}

pub fn current_kstack_top() -> usize {
    // current_task().unwrap().kstack.get_top()
    current_task().unwrap().kstack.get_position().1
}

pub fn schedule(switched_task_cx_ptr: *mut KContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    // info!("schedule: {:#x?}", unsafe { switched_task_cx_ptr.as_mut().unwrap() });
    drop(processor);
    // FIXME: Switch context
    // unsafe {
    //     __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    // }
    unsafe {
        context_switch_pt(switched_task_cx_ptr, idle_task_cx_ptr, kernel_page_table());
    }
}
