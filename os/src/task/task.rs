//!Implementation of [`TaskControlBlock`]
use super::{current_task_tcb};
use alloc::sync::{Arc};
use polyhal::pagetable::PageTable;
use crate::config::KERNEL_STACK_SIZE;
use crate::mm::MemorySet;
use core::mem::size_of;
use polyhal::{
    read_current_tp, run_user_task, KContext, KContextArgs, TrapFrame, TrapFrameArgs,
};

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

fn task_entry() {
    let task = current_task_tcb();
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

pub struct TaskControlBlock {
    pub trap_cx: TrapFrame,
    pub base_size: usize,
    pub task_cx: KContext,
    pub task_status: TaskStatus,
    pub kernel_stack: KernelStack,
    pub memory_set: MemorySet,
}

impl TaskControlBlock {
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
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        // alloc a kernel stack in kernel space
        let kstack = KernelStack::new();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            trap_cx: TrapFrame::new(),
            base_size: user_sp,
            task_cx: blank_kcontext(kstack.get_position().1),
            task_status: TaskStatus::Ready,
            memory_set,
            kernel_stack: kstack,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        trap_cx[TrapFrameArgs::SEPC] = entry_point;
        trap_cx[TrapFrameArgs::SP] = user_sp;
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
