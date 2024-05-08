mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;
mod vpn_range;

pub use frame_allocator::init_frame_allocator;
pub use frame_allocator::{frame_alloc, frame_alloc_persist, frame_dealloc, FrameTracker};
pub use heap_allocator::init_heap;
pub use memory_set::MemorySet;
use alloc::vec::Vec;
pub use page_table::{translated_byte_buffer, translated_ref, translated_refmut, translated_str};

pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}