use super::{current_task, SignalActions};
use super::{pid_alloc, PidHandle, SignalFlags};
use crate::config::KERNEL_STACK_SIZE;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{translated_refmut, MemorySet};
use crate::sync::UPSafeCell;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use polyhal::pagetable::PageTable;
use polyhal::{
    read_current_tp, run_user_task, KContext, KContextArgs, TrapFrame, TrapFrameArgs,
};
use core::cell::RefMut;
use core::mem::size_of;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}
use log::*;
impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        trace!("drop task control block");
    }
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
    pub memory_set: MemorySet,
    pub kernel_stack: KernelStack,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub signals: SignalFlags,
    pub signal_mask: SignalFlags,
    // the signal which is being handling
    pub handling_sig: isize,
    // Signal actions
    pub signal_actions: SignalActions,
    // if the task is killed
    pub killed: bool,
    // if the task is frozen by a signal
    pub frozen: bool,
    pub trap_ctx_backup: Option<TrapFrame>,
}

impl TaskControlBlockInner {
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
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

fn task_entry() {
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
        let task_control_block = Self {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx: TrapFrame::new(),
                    base_size: user_sp,
                    task_cx: blank_kcontext(kstack.get_position().1), // Set task_cx's Kernel Stack Top
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    signals: SignalFlags::empty(),
                    signal_mask: SignalFlags::empty(),
                    handling_sig: -1,
                    signal_actions: SignalActions::default(),
                    killed: false,
                    frozen: false,
                    trap_ctx_backup: None,
                    kernel_stack: kstack,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx[TrapFrameArgs::SEPC] = entry_point;
        trap_cx[TrapFrameArgs::SP] = user_sp;
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, mut user_sp, entry_point) = MemorySet::from_elf(elf_data);
        memory_set.activate();
        // push arguments on user stack
        user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| {
                translated_refmut(
                    memory_set.token(),
                    (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize,
                )
            })
            .collect();
        *argv[args.len()] = 0;
        for i in 0..args.len() {
            user_sp -= args[i].len() + 1;
            *argv[i] = user_sp;
            let mut p = user_sp;
            for c in args[i].as_bytes() {
                *translated_refmut(memory_set.token(), p as *mut u8) = *c;
                p += 1;
            }
            *translated_refmut(memory_set.token(), p as *mut u8) = 0;
        }
        // make the user_sp aligned to 8B for k210 platform
        user_sp -= user_sp % core::mem::size_of::<usize>();
        memory_set.activate();

        // **** access current TCB exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        // FIXME: This is a temporary solution
        inner.trap_cx = TrapFrame::new();
        // initialize trap_cx
        let mut trap_cx = TrapFrame::new();
        trap_cx[TrapFrameArgs::SEPC] = entry_point;
        trap_cx[TrapFrameArgs::SP] = user_sp;
        trap_cx[TrapFrameArgs::ARG0] = args.len();
        trap_cx[TrapFrameArgs::ARG1] = argv_base;
        *inner.get_trap_cx() = trap_cx;
        // **** release current PCB
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- hold parent PCB lock
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);

        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kstack = KernelStack::new();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx: parent_inner.trap_cx.clone(),
                    base_size: parent_inner.base_size,
                    task_cx: blank_kcontext(kstack.get_position().1), // Set task_cx's Kernel Stack Top
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    signals: SignalFlags::empty(),
                    // inherit the signal_mask and signal_action
                    signal_mask: parent_inner.signal_mask,
                    handling_sig: -1,
                    signal_actions: parent_inner.signal_actions.clone(),
                    killed: false,
                    frozen: false,
                    trap_ctx_backup: None,
                    kernel_stack: kstack,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access child PCB exclusively
        task_control_block
        // **** release child PCB
        // ---- release parent PCB
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
