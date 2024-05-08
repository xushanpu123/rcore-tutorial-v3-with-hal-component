use core::ptr::NonNull;

use super::BlockDevice;
use crate::mm::{frame_alloc, frame_dealloc, FrameTracker};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use polyhal::addr::{PhysAddr, PhysPage};
use polyhal::VIRT_ADDR_START;
use lazy_static::*;
use log::debug;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal};

#[allow(unused)]
#[cfg(target_arch = "riscv64")]
const VIRTIO0: usize = 0x10001000;

#[cfg(target_arch = "aarch64")]
const VIRTIO0: usize = 0xa00_0000;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<VirtioHal, MmioTransport>>);

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

unsafe impl Sync for VirtIOBlock {}
unsafe impl Send for VirtIOBlock {}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .exclusive_access()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .exclusive_access()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal, MmioTransport>::new(
                    MmioTransport::new(NonNull::new_unchecked(
                        (VIRTIO0 | VIRT_ADDR_START) as *mut VirtIOHeader,
                    ))
                    .expect("this is not a valid virtio device"),
                )
                .unwrap(),
            ))
        }
    }
}

pub struct VirtioHal;

// unsafe impl Hal for VirtioHal {
//     fn dma_alloc(pages: usize, direction: BufferDirection) -> usize {
//         let mut ppn_base = PhysPage::new(0);
//         for i in 0..pages {
//             let frame = frame_alloc().unwrap();
//             debug!("alloc paddr: {:?}", frame);
//             if i == 0 {
//                 ppn_base = frame.ppn;
//             }
//             assert_eq!(frame.ppn.as_num(), ppn_base.as_num() + i);
//             QUEUE_FRAMES.exclusive_access().push(frame);
//         }
//         let pa: PhysAddr = ppn_base.into();
//         pa.addr()
//     }

//     fn dma_dealloc(pa: usize, pages: usize) -> i32 {
//         debug!("dealloc paddr: {:?}", pa);
//         let pa = PhysAddr::new(pa);
//         let mut ppn_base: PhysPage = pa.into();
//         for _ in 0..pages {
//             frame_dealloc(ppn_base);
//             ppn_base = ppn_base + 1;
//         }
//         0
//     }

//     unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, size: usize) -> core::ptr::NonNull<u8> {

//     }

//     unsafe fn share(buffer: core::ptr::NonNull<[u8]>, direction: virtio_drivers::BufferDirection) -> virtio_drivers::PhysAddr {
//         todo!()
//     }

//     unsafe fn unshare(paddr: virtio_drivers::PhysAddr, buffer: core::ptr::NonNull<[u8]>, direction: virtio_drivers::BufferDirection) {
//         todo!()
//     }
// }

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (usize, NonNull<u8>) {
        let mut ppn_base = PhysPage::new(0);
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            debug!("alloc paddr: {:?}", frame);
            if i == 0 {
                ppn_base = frame.ppn
            };
            assert_eq!(frame.ppn.as_num(), ppn_base.as_num() + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: usize = ppn_base.to_addr();
        unsafe {
            (
                pa,
                NonNull::new_unchecked((pa | VIRT_ADDR_START) as *mut u8),
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
