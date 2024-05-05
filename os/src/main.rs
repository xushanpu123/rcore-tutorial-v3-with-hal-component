//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.

//#![deny(missing_docs)]
//#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate polyhal;
extern crate alloc;
#[macro_use]
extern crate bitflags;
use core::arch::global_asm;
use buddy_system_allocator::LockedHeap;
use log::info;
use polyhal::pagetable::PageTableWrapper;

#[global_allocator]            
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
//use log::*;
#[macro_use]
mod console;
pub mod batch;
pub mod frame_allocater;
pub mod heap_allocator;
mod lang_items;
mod logging;
mod sync;
pub mod syscall;
pub mod config;
use crate::syscall::syscall;
use crate::batch::*;
pub use crate::frame_allocater::*;
use polyhal::{get_mem_areas, PageAlloc, TrapFrame, TrapFrameArgs, TrapType};
use polyhal::addr::PhysPage;
use polyhal::TrapType::*;
pub use heap_allocator::init_heap;
global_asm!(include_str!("link_app.S"));

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    #[inline]
    fn alloc(&self) -> PhysPage {
        frame_alloc_persist().expect("can't find memory page")
    }

    #[inline]
    fn dealloc(&self, ppn: PhysPage) {
        frame_dealloc(ppn)
    }
}

/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    match trap_type {
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
            run_next_app();
        }
        IllegalInstruction(_) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }
        Time => {
            
        }
        _ => {
            panic!("unsuspended trap type: {:?}", trap_type);
        }
    }
}

/// the rust entry-point of os
#[polyhal::arch_entry]
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    init_heap();
    logging::init(Some("trace"));
    polyhal::init(&PageAllocImpl);
    get_mem_areas().into_iter().for_each(|(start, size)| {
        info!("frame alloocator add frame {:#x} - {:#x}", start, start + size);
        init_frame_allocator(start, start + size);
    });
    let new_page_table = PageTableWrapper::alloc();
    new_page_table.change();
    batch::init();
    batch::run_next_app();
}
