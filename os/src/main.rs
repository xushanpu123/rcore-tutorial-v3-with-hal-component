//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`task`]: Task management
//! - [`syscall`]: System call handling and implementation
//! - [`mm`]: Address map using SV39
//! - [`sync`]:Wrap a static data structure inside it so that we are able to access it without any `unsafe`.
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`task::run_tasks()`] and for the first time go to
//! userspace.

//#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate polyhal;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod logging;
mod timer;
#[path="boards/qemu.rs"]
mod board;
mod loader;
pub mod mm;
pub mod sync;
pub mod syscall;
pub mod task;

use crate::syscall::syscall;
use crate::task::{suspend_current_and_run_next, exit_current_and_run_next};
use polyhal::{get_mem_areas, PageAlloc, TrapFrame, TrapFrameArgs, TrapType};
use polyhal::addr::PhysPage;
use polyhal::TrapType::*;
use log::*;

use core::arch::global_asm;

global_asm!(include_str!("link_app.S"));

#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    match trap_type {
        Breakpoint => return,
        UserEnvCall => {
            // jump to next instruction anyway
            ctx.syscall_ok();
            let args = ctx.args();
            // get system call return value
            // info!("syscall: {}", ctx[TrapFrameArgs::SYSCALL]);

            let result = syscall(ctx[TrapFrameArgs::SYSCALL], [args[0], args[1], args[2]]);
            // cx is changed during sys_exec, so we have to call it again
            ctx[TrapFrameArgs::RET] = result as usize;
        }
        StorePageFault(_paddr) | LoadPageFault(_paddr) | InstructionPageFault(_paddr) => {
            println!("[kernel] PageFault in application, kernel killed it. paddr={:x}",_paddr);
            exit_current_and_run_next(-2);
        }
        IllegalInstruction(_) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next(-2);
        }
        Time => {
            suspend_current_and_run_next();
        }
        _ => {
            panic!("unsuspended trap type: {:?}", trap_type);
        }
    }
}

#[polyhal::arch_entry]
pub fn main(hartid: usize){
    trace!("ch5 main start: hartid: {}", hartid);
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    mm::init_heap();
    logging::init(Some("info"));
    info!("[kernel] init logging success!");
    polyhal::init(&PageAllocImpl);
    get_mem_areas().into_iter().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        mm::init_frame_allocator(start, start + size);
    });

    task::add_initproc();
    println!("after initproc!");
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in ch5 rust main!");
}

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    #[inline]
    fn alloc(&self) -> PhysPage {
        mm::frame_alloc_page_with_clear().expect("failed to alloc page")
    }

    #[inline]
    fn dealloc(&self, ppn: PhysPage) {
        mm::frame_dealloc(ppn)
    }
}
