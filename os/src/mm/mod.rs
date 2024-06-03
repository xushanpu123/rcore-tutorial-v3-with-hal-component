mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use address::VPNRange;
pub use frame_allocator::{frame_alloc, frame_alloc_persist, frame_dealloc, FrameTracker, init_frame_allocator};
pub use memory_set::{MapPermission, MemorySet};

pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str,
};

pub fn init() {
    heap_allocator::init_heap();
}
