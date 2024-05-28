//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.
use polyhal::addr::{PhysAddr, PhysPage};

use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;
use polyhal::{PAGE_SIZE, VIRT_ADDR_START};
use core::mem::size_of;

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    ///
    pub ppn: PhysPage,
}

impl FrameTracker {
    ///Create an empty `FrameTracker`
    pub fn new(ppn: PhysPage) -> Self {
        // page cleaning
        ppn.drop_clear();
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:?}", self.ppn))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPage>;
    fn dealloc(&mut self, ppn: PhysPage);
}
/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPage, r: PhysPage) {
        self.current = l.as_num();
        self.end = r.as_num();
        println!("last {} Physical Frames.", self.end - self.current);
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPage> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPage) {
        let ppn = ppn.as_num();
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}
/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator(mm_start: usize, mm_end: usize) {
    extern "C" {
        fn end();
    }
    let phys_end = end as usize;
    if phys_end >= mm_start && phys_end < mm_end {
        unsafe {
            core::slice::from_raw_parts_mut(
                phys_end as *mut u128,
                (mm_end - phys_end) / size_of::<u128>(),
            )
            .fill(0);
        }
        let start = ((phys_end + 0xfff) / PAGE_SIZE * PAGE_SIZE) & (!VIRT_ADDR_START);
        FRAME_ALLOCATOR.exclusive_access().init(
            PhysAddr::new(start).into(),
            PhysAddr::new(mm_end & (!VIRT_ADDR_START)).into(),
        );
    }
}
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// doc
pub fn frame_alloc_page_with_clear() -> Option<PhysPage> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .inspect(|x| x.drop_clear())
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPage) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
