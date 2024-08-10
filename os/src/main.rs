//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from #[polyhal::arch_entry] that is [`main()`] , polyhal helps finish 
//! all initialization work.
//! 
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use buddy_system_allocator::LockedHeap;
use polyhal::instruction::Instruction;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();


#[macro_use]
mod console;
mod lang_items;
mod logging;

//The entry point
#[polyhal::arch_entry]
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    Instruction::shutdown();
}
