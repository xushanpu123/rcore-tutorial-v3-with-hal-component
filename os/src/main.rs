//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use buddy_system_allocator::LockedHeap;
use polyhal::TrapFrame;
use polyhal::TrapType;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();


#[macro_use]
mod console;
mod lang_items;
mod logging;

/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(_ctx: &mut TrapFrame, _trap_type: TrapType) {

}
//The entry point
#[polyhal::arch_entry]
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    polyhal::shutdown();
}
