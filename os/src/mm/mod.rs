//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;
mod vpn_range;

pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::{MapPermission, MemorySet};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str, translated_ref};
pub use heap_allocator::init_heap;
pub use frame_allocator::{frame_alloc_page_with_clear, frame_dealloc, init_frame_allocator};