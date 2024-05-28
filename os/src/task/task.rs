//!Implementation of [`TaskControlBlock`]
use super::{pid_alloc, PidHandle, current_task};
use crate::sync::UPSafeCell;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use polyhal::pagetable::PageTable;
use core::cell::RefMut;
use crate::config::KERNEL_STACK_SIZE;
use crate::mm::MemorySet;
use core::mem::size_of;
use polyhal::{
    read_current_tp, run_user_task, KContext, KContextArgs, TrapFrame, TrapFrameArgs,
};
use log::*;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct KernelStack {
    inner: Arc<[u128; KERNEL_STACK_SIZE / size_of::<u128>()]>,
}

impl KernelStack {
    pub fn new() -> Self {
        Self {
            inner: Arc::new([0u128; KERNEL_STACK_SIZE / size_of::<u128>()]),
        }
    }

    pub fn get_position(&self) -> (usize, usize) {
        let bottom = self.inner.as_ptr() as usize;
        (bottom, bottom + KERNEL_STACK_SIZE)
    }
}

pub struct TaskControlBlockInner {
    pub trap_cx: TrapFrame,
    pub base_size: usize,
    pub task_cx: KContext,
    pub task_status: TaskStatus,
    pub kernel_stack: KernelStack,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    /*
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    */
    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        let paddr = &self.trap_cx as *const TrapFrame as usize as *mut TrapFrame;
        // let paddr: PhysAddr = self.trap_cx.into();
        // unsafe { paddr.get_mut_ptr::<TrapFrame>().as_mut().unwrap() }
        unsafe { paddr.as_mut().unwrap() }
    }
    pub fn get_user_token(&self) -> PageTable {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
}

fn task_entry() {
    trace!("os::task::task_entry");
    let task = current_task()
        .unwrap()
        .inner
        .exclusive_access()
        .get_trap_cx() as *mut TrapFrame;
    // run_user_task_forever(unsafe { task.as_mut().unwrap() })
    let ctx_mut = unsafe { task.as_mut().unwrap() };
    loop {
        run_user_task(ctx_mut);
    }
}

fn blank_kcontext(ksp: usize) -> KContext {
    let mut kcx = KContext::blank();
    kcx[KContextArgs::KPC] = task_entry as usize;
    kcx[KContextArgs::KSP] = ksp;
    kcx[KContextArgs::KTP] = read_current_tp();
    kcx
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kstack = KernelStack::new();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx: TrapFrame::new(),
                    base_size: user_sp,
                    task_cx: blank_kcontext(kstack.get_position().1),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    kernel_stack: kstack,
                    exit_code: 0,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx[TrapFrameArgs::SEPC] = entry_point;
        trap_cx[TrapFrameArgs::SP] = user_sp;
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        memory_set.activate();
        
        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx = TrapFrame::new();
        // initialize base_size
        let mut trap_cx = TrapFrame::new();
        trap_cx[TrapFrameArgs::SEPC] = entry_point;
        trap_cx[TrapFrameArgs::SP] = user_sp;
        *inner.get_trap_cx() = trap_cx;
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kstack = KernelStack::new();
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx: parent_inner.trap_cx.clone(),
                    base_size: parent_inner.base_size,
                    task_cx: blank_kcontext(kstack.get_position().1),
                    task_status: TaskStatus::Ready,
                    kernel_stack: kstack,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        task_control_block
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
