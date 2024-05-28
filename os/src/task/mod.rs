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

use crate::loader::{get_num_app,get_app_data};
use crate::polyhal::shutdown;
use crate::sync::UPSafeCell;
use log::info;
use alloc::vec::Vec;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};
use polyhal::{KContext, TrapFrame};
use polyhal::{pagetable::PageTable};
use polyhal::context_switch_pt;

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
        for i in 0..num_app{
            tasks.push(TaskControlBlock::new(get_app_data(i)));}
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
        let token = task0.memory_set.token();
        drop(inner);
        let mut _unused = KContext::blank();
        // before this, we should drop local variables that must be dropped manually
        info!("context_switch before!");
        unsafe {
            context_switch_pt(&mut _unused as *mut KContext, next_task_cx_ptr, token);
        }
        info!("context_switch after!");
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
            let token = inner.tasks[next].memory_set.token();
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                context_switch_pt(current_task_cx_ptr, next_task_cx_ptr, token);
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

pub fn current_task_tcb() -> *mut TrapFrame {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let ret = inner.tasks[inner.current_task].get_trap_cx() as *mut TrapFrame;
    ret
}

pub fn current_user_token() -> PageTable {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let ret = inner.tasks[inner.current_task].get_user_token();
    ret
}
