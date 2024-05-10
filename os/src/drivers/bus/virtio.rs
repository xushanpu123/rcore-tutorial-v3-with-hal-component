use crate::mm::{
    frame_alloc_more, frame_dealloc, FrameTracker};
use crate::sync::UPIntrFreeCell;
use crate::task::current_process;
use alloc::vec::Vec;
use lazy_static::*;
use polyhal::PAGE_SIZE;
use virtio_drivers::Hal;
use polyhal::addr::PhysAddr;
use polyhal::addr::PhysPage;
use polyhal::addr::VirtAddr;

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<FrameTracker>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        pa.addr()
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::new(pa);
        let mut ppn_base: PhysPage = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base = ppn_base + 1
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr + 0xffff_ffc0_0000_0000
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        vaddr - 0xffff_ffc0_0000_0000
    }
}
