//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.


#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM,PAGE_SIZE};
use crate::loader::{get_num_app,get_ksp,get_base_i};
use crate::polyhal::shutdown;
use crate::sync::UPSafeCell;
use alloc::boxed::Box;
use log::info;
use alloc::vec::Vec;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};
use polyhal::{
    read_current_tp, run_user_task, KContext, KContextArgs, TrapFrame, TrapFrameArgs,
};
use polyhal::context_switch;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut kcx = KContext::blank();
        let mut tasks = Vec::new();
        for i in 0..MAX_APP_NUM{
            tasks.push(TaskControlBlock {
                task_cx: KContext::blank(),
                task_status: TaskStatus::UnInit,
        });}
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_status = TaskStatus::Ready;
            task.task_cx[KContextArgs::KPC] = task_entry as usize;
            task.task_cx[KContextArgs::KSP] = get_ksp(i);
            task.task_cx[KContextArgs::KTP] = read_current_tp();          
            }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}


impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const KContext;
        drop(inner);
        let mut _unused = KContext::blank();
        // before this, we should drop local variables that must be dropped manually
        info!("context_switch before!");
        unsafe {
            context_switch(&mut _unused as *mut KContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut KContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const KContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                context_switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            println!("All applications completed!");
            shutdown();
        }
    }
    fn current_task(&self)->usize{
        self.inner.exclusive_access().current_task
    }
}

/// run first task
pub fn run_first_task() {
    println!("123");
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}


fn task_entry() {
    let app_id = TASK_MANAGER.current_task();
    // 这里的 Box::new 是为了保证 TrapFrame 能够被被正确的对齐
    // 如果没有 Box::new 在使用 x86_64 的时候会由于地址没有对齐导致 #GP 错误
    let mut trap_cx = Box::new(TrapFrame::new());
    trap_cx[TrapFrameArgs::SEPC] = get_base_i(app_id);
    trap_cx[TrapFrameArgs::SP] = 0x1_8000_0000 + (app_id+1)*PAGE_SIZE;
    let ctx_mut = trap_cx.as_mut();
    loop {
        run_user_task(ctx_mut);
    }
    panic!("Unreachable in batch::run_current_app!");
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
