#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod net;
mod sync;
mod syscall;
mod task;
mod timer;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

use crate::task::{current_process, exit_current_and_run_next};
use crate::{syscall::syscall, task::check_signals_of_current};
use lazy_static::*;
use log::*;
use polyhal::debug::DebugConsole;
use polyhal::irq::IRQ;
use polyhal::{
    addr::PhysPage, get_cpu_num, get_fdt, get_mem_areas, PageAlloc, TrapFrame, TrapFrameArgs,
    TrapType,
};
use polyhal::{debug, TrapType::*};
use polyhal::TrapType::*;
use sync::UPIntrFreeCell;
use task::{current_add_signal, current_task, suspend_current_and_run_next, SignalFlags};

use crate::drivers::block::BLOCK_DEVICE;
#[cfg(target_arch = "riscv64")]
use crate::drivers::plic::{IntrTargetPriority, PLIC};

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    match trap_type {
        Breakpoint => return,
        UserEnvCall => {
            // jump to next instruction anyway
            ctx.syscall_ok();
            let args = ctx.args();
            // get system call return value

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
            info!("stval: {:#x}", _paddr);
            current_add_signal(SignalFlags::SIGSEGV);
        }
        IllegalInstruction(_) => {
            log::error!("Illegal instruction");
            current_add_signal(SignalFlags::SIGILL);
        }
        Time => {
            // log::info!("Time from user: {}", ctx.from_user());
            if !ctx.from_user() || current_task().is_none() {
                return;
            }
            suspend_current_and_run_next();
        }
        Irq(irq) => {
            log::info!("irq: {}", irq.irq_num());
            //   VirtIO Block  IRQ: 266 & 0xff = 10;
            //   VirtIO GPU  IRQ: 267 & 0xff = 11;
            //   VirtIO Network  IRQ: 267
            //   VirtIO Input  IRQ: 266
            //   VirtIO Input  IRQ: 266
            #[cfg(target_arch = "x86_64")]
            {
                match irq.irq_num() {
                    0x4 => {
                        log::info!("COM Input: {:?}", DebugConsole::getchar());
                    }
                    10 => {
                        BLOCK_DEVICE.handle_irq();
                        
                    },
                    _=>{}
                };
            }
            #[cfg(target_arch = "aarch64")]
            {
                match irq.irq_num() {
                    0x4f => BLOCK_DEVICE.handle_irq(),
                    0x4c => KEYBOARD_DEVICE.handle_irq(),
                    0x4b => MOUSE_DEVICE.handle_irq(),
                    // aarch64 uart interrupt
                    // 0x21 => {
                    //     // let key = DebugConsole::getchar().unwrap();
                    //     // log::info!("input key: {}", key);
                    //     irq.ack();
                    // }
                    _ => panic!("unsupported IRQ {}", irq.irq_num()),
                }
            }
            irq.ack();
        }
        SupervisorExternal => {
            #[cfg(target_arch = "riscv64")]
            {
                use board::VIRT_PLIC;
                let mut plic: PLIC = unsafe { PLIC::new(VIRT_PLIC) };
                let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
                log::trace!("entry SupervisorExternal, intr_src_id: {}", intr_src_id);
                match intr_src_id {
                    5 => KEYBOARD_DEVICE.handle_irq(),
                    6 => MOUSE_DEVICE.handle_irq(),
                    8 => BLOCK_DEVICE.handle_irq(),
                    // 10 => UART.handle_irq(),
                    _ => panic!("unsupported IRQ {}", intr_src_id),
                }
                plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
            }
        }
        _ => {
            warn!("unsuspended trap type: {:?}", trap_type);
        }
    }
    if let Some((errno, msg)) = check_signals_of_current() {
        println!("[kernel] {}", msg);
        // panic!("end");
        exit_current_and_run_next(errno);
    }
}

#[polyhal::arch_entry]
pub fn rust_main(_hartid: usize) -> ! {
    mm::init();
    logging::init(option_env!("LOG"));
    polyhal::init(&PageAllocImpl);
    get_mem_areas().into_iter().for_each(|(start, size)| {
        mm::init_frame_allocator(start, start + size);
    });
    UART.init();
    IRQ::int_disable();
    println!("KERN: init plic");
    println!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    println!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    println!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    board::device_init();
    fs::list_apps();
    task::add_initproc();
    log::info!("open nonblock block device read");
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    IRQ::int_enable();
    // *DEV_NON_BLOCKING_ACCESS.exclusive_access() = false;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    #[inline]
    fn alloc(&self) -> PhysPage {
        let res = mm::frame_alloc_persist().expect("can't find memory page");
        res
    }

    #[inline]
    fn dealloc(&self, ppn: PhysPage) {
        mm::frame_dealloc(ppn)
    }
}
