use core::ptr::NonNull;

use crate::mm::{frame_alloc_more, frame_dealloc, FrameTracker};
use crate::sync::UPIntrFreeCell;
use alloc::vec::Vec;
use lazy_static::*;
use polyhal::addr::PhysAddr;
use polyhal::addr::PhysPage;
use polyhal::consts::VIRT_ADDR_START;
use virtio_drivers::{BufferDirection, Hal};

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<FrameTracker>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

// impl Hal for VirtioHal {
//     fn dma_alloc(pages: usize) -> usize {
//         let trakcers = frame_alloc_more(pages);
//         let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
//         QUEUE_FRAMES
//             .exclusive_access()
//             .append(&mut trakcers.unwrap());
//         let pa: PhysAddr = ppn_base.into();
//         pa.addr()
//     }

//     fn dma_dealloc(pa: usize, pages: usize) -> i32 {
//         let pa = PhysAddr::new(pa);
//         let mut ppn_base: PhysPage = pa.into();
//         for _ in 0..pages {
//             frame_dealloc(ppn_base);
//             ppn_base = ppn_base + 1
//         }
//         0
//     }

//     fn phys_to_virt(addr: usize) -> usize {
//         addr + VIRT_ADDR_START
//     }

//     fn virt_to_phys(vaddr: usize) -> usize {
//         vaddr - VIRT_ADDR_START
//     }
// }

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (usize, NonNull<u8>) {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        unsafe {
            (
                pa.addr(),
                NonNull::new_unchecked((pa.addr() | VIRT_ADDR_START) as *mut u8),
            )
        }
    }

    unsafe fn dma_dealloc(paddr: usize, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        // trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        log::error!("dealloc paddr: {:?}", paddr);
        let pa = PhysAddr::new(paddr);
        let mut ppn_base: PhysPage = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base = ppn_base + 1;
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: usize, _size: usize) -> NonNull<u8> {
        NonNull::new((usize::from(paddr) | VIRT_ADDR_START) as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> usize {
        buffer.as_ptr() as *mut u8 as usize - VIRT_ADDR_START
        // let pt = PageTable::current();
        // let paddr = pt.translate(VirtAddr::new(buffer.as_ptr() as *const u8 as usize)).expect("can't find vaddr").0;
        // paddr.addr()
    }

    unsafe fn unshare(_paddr: usize, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}
