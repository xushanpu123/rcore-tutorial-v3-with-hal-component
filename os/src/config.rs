//! Constants used in rCore
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

pub const PAGE_SIZE: usize = 0x1000;
#[allow(unused)]
pub const PAGE_SIZE_BITS: usize = 0xc;
#[allow(unused)]
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
#[allow(unused)]
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
#[allow(unused)]
pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};
