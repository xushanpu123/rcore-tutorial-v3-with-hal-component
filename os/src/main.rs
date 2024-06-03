#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

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

use crate::{syscall::syscall, task::check_signals_of_current};
use crate::task::{current_task, exit_current_and_run_next};
use polyhal::TrapType::*;
use log::{info, warn};
use polyhal::{addr::PhysPage, get_mem_areas, PageAlloc, TrapFrame, TrapFrameArgs, TrapType};
use task::{current_add_signal, suspend_current_and_run_next, SignalFlags};

use crate::mm::init_frame_allocator;

// #[no_mangle]
// pub fn trap_handler() -> ! {
//     set_kernel_trap_entry();
//     let scause = scause::read();
//     let stval = stval::read();
//     match scause.cause() {
//         Trap::Exception(Exception::UserEnvCall) => {
//             // jump to next instruction anyway
//             let mut cx = current_trap_cx();
//             cx.sepc += 4;
//             // get system call return value
//             let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
//             // cx is changed during sys_exec, so we have to call it again
//             cx = current_trap_cx();
//             cx.x[10] = result as usize;
//         }
//         Trap::Exception(Exception::StoreFault)
//         | Trap::Exception(Exception::StorePageFault)
//         | Trap::Exception(Exception::InstructionFault)
//         | Trap::Exception(Exception::InstructionPageFault)
//         | Trap::Exception(Exception::LoadFault)
//         | Trap::Exception(Exception::LoadPageFault) => {
//             /*
//             println!(
//                 "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
//                 scause.cause(),
//                 stval,
//                 current_trap_cx().sepc,
//             );
//             */
//             current_add_signal(SignalFlags::SIGSEGV);
//         }
//         Trap::Exception(Exception::IllegalInstruction) => {
//             current_add_signal(SignalFlags::SIGILL);
//         }
//         Trap::Interrupt(Interrupt::SupervisorTimer) => {
//             set_next_trigger();
//             check_timer();
//             suspend_current_and_run_next();
//         }
//         _ => {
//             panic!(
//                 "Unsupported trap {:?}, stval = {:#x}!",
//                 scause.cause(),
//                 stval
//             );
//         }
//     }
//     // check signals
//     if let Some((errno, msg)) = check_signals_of_current() {
//         println!("[kernel] {}", msg);
//         exit_current_and_run_next(errno);
//     }
//     trap_return();
// }


/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    log::trace!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    // info!("current_task id: {}", current_task().is_some());
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
            
            info!(
                "[kernel] in application, bad addr = {:#x}, ctx: {:#x?} kernel killed it.",
                //scause.cause(),
                _paddr,
                ctx
                //current_trap_cx().sepc,
            );
            
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
    // handle_signals();

    // // check error signals (if error then exit)
    // if let Some((errno, msg)) = check_signals_error_of_current() {
    //     println!("[kernel] {}", msg);
    //     exit_current_and_run_next(errno);
    // }
    if let Some((errno, msg)) = check_signals_of_current() {
        println!("[kernel] {}", msg);
        // panic!("end");
        exit_current_and_run_next(errno);
    }
}
#[polyhal::arch_entry]
pub fn rust_main() -> ! {
    println!("[kernel] Hello, world!");
    mm::init();
    logging::init(Some("debug"));
    println!("init logging");
    polyhal::init(&PageAllocImpl);
    get_mem_areas().into_iter().for_each(|(start, size)| {
        init_frame_allocator(start, start + size);
    });
    // mm::remap_test();

    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
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
