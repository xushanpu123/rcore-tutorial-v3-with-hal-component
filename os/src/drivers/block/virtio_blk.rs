use core::ptr::NonNull;

use super::BlockDevice;
use crate::drivers::bus::virtio::VirtioHal;
use crate::sync::{Condvar, UPIntrFreeCell};
use crate::task::schedule;
use crate::DEV_NON_BLOCKING_ACCESS;
use alloc::collections::BTreeMap;
use polyhal::irq::IRQ;
use polyhal::VIRT_ADDR_START;
use virtio_drivers::device::blk::{BlkReq, BlkResp, RespStatus, VirtIOBlk};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
const VIRTIO0: usize = 0x10008000 + VIRT_ADDR_START;

#[cfg(target_arch = "aarch64")]
const VIRTIO0: usize = 0xa00_3e00 + VIRT_ADDR_START;

pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    condvars: BTreeMap<u16, Condvar>,
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let nb = *DEV_NON_BLOCKING_ACCESS.exclusive_access();
        if nb {
            let mut request = BlkReq::default();
            let mut resp = BlkResp::default();
            let mut token = 0;
            let task_cx_ptr = self.virtio_blk.exclusive_session(|blk| {
                token = unsafe { blk.read_blocks_nb(block_id, &mut request, buf, &mut resp).unwrap() };
                self.condvars.get(&token).unwrap().wait_no_sched()
            });
            schedule(task_cx_ptr);
            self.virtio_blk.exclusive_session(|blk| unsafe {
                blk.complete_read_blocks(token, &request, buf, &mut resp).unwrap();
            });
            assert_eq!(
                resp.status(),
                RespStatus::OK,
                "Error when reading VirtIOBlk"
            );
        } else {
            self.virtio_blk
                .exclusive_access()
                .read_blocks(block_id, buf)
                .expect("Error when reading VirtIOBlk");
        }
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let nb = *DEV_NON_BLOCKING_ACCESS.exclusive_access();
        if nb {
            let mut request = BlkReq::default();
            let mut resp = BlkResp::default();
            let mut token = 0;
            let task_cx_ptr = self.virtio_blk.exclusive_session(|blk| {
                token = unsafe { blk.write_blocks_nb(block_id, &mut request, buf, &mut resp).unwrap() };
                self.condvars.get(&token).unwrap().wait_no_sched()
            });
            schedule(task_cx_ptr);
            self.virtio_blk.exclusive_session(|blk| unsafe{
                blk.complete_write_blocks(token, &request, buf, &mut resp).unwrap();
            });
            assert_eq!(
                resp.status(),
                RespStatus::OK,
                "Error when writing VirtIOBlk"
            );
        } else {
            self.virtio_blk
                .exclusive_access()
                .write_blocks(block_id, buf)
                .expect("Error when writing VirtIOBlk");
        }
    }
    fn handle_irq(&self) {
        self.virtio_blk.exclusive_session(|blk| {
            // if *DEV_NON_BLOCKING_ACCESS.exclusive_access() {
            //     blk.ack_interrupt();
            // }
            blk.ack_interrupt();
            if let Some(token) = blk.peek_used() {
                log::info!("peek token: {:?}", token);
                self.condvars.get(&token).unwrap().signal();
            }
        });
    }
}

impl VirtIOBlock {
    pub fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            super::super::bus::pci::init();
            todo!("init pci");
        }
        // let virtio_blk = unsafe {
        //     UPIntrFreeCell::new(
        //         VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
        //     )
        // };

        #[cfg(not(target_arch = "x86_64"))]
        {
            let virtio_blk = unsafe {
                UPIntrFreeCell::new(
                    VirtIOBlk::<VirtioHal, MmioTransport>::new(
                        MmioTransport::new(NonNull::new_unchecked(
                            (VIRTIO0 | VIRT_ADDR_START) as *mut VirtIOHeader,
                        )).unwrap()
                    ).unwrap(),
                )
            };
            #[cfg(target_arch = "aarch64")]
            IRQ::irq_enable(0x4f);
            let mut condvars = BTreeMap::new();
            let channels = virtio_blk.exclusive_access().virt_queue_size();
            for i in 0..channels {
                let condvar = Condvar::new();
                condvars.insert(i, condvar);
            }
            Self {
                virtio_blk,
                condvars,
            }
        }
    }
}
