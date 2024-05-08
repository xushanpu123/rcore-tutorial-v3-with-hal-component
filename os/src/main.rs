#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use crate::{
    syscall::syscall,
    task::{
        check_signals_error_of_current, current_add_signal, exit_current_and_run_next,
        handle_signals, suspend_current_and_run_next, SignalFlags,
    },
};
use polyhal::{get_mem_areas, PageAlloc, TrapFrame, TrapFrameArgs, TrapType};
// use polyhal::api::ArchInterface;
use polyhal::addr::PhysPage;
use log::warn;
use polyhal::TrapType::*;
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod sync;
mod syscall;
mod task;

pub struct ArchInterfaceImpl;

/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
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
            /*
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            */
            current_add_signal(SignalFlags::SIGSEGV);
        }
        IllegalInstruction(_) => {
            current_add_signal(SignalFlags::SIGILL);
        }
        Time => {
            suspend_current_and_run_next();
        }
        _ => {
            warn!("unsuspended trap type: {:?}", trap_type);
        }
    }
    // handle signals (handle the sent signal)
    // println!("[K] trap_handler:: handle_signals");
    handle_signals();

    // check error signals (if error then exit)
    if let Some((errno, msg)) = check_signals_error_of_current() {
        println!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }
}

#[polyhal::arch_entry]
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    mm::init_heap();
    logging::init(Some("trace"));
    println!("init logging");
    polyhal::init_interrupt();

    polyhal::init(&PageAllocImpl);
    get_mem_areas().into_iter().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        mm::init_frame_allocator(start, start + size);
    });

    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in main function of rCore Tutorial kernel!");
}

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    #[inline]
    fn alloc(&self) -> PhysPage {
        mm::frame_alloc_persist().expect("can't find memory page")
    }

    #[inline]
    fn dealloc(&self, ppn: PhysPage) {
        mm::frame_dealloc(ppn)
    }
}
