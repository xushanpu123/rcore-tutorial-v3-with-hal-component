use core::any::Any;
use core::ptr::NonNull;

use crate::drivers::virtio::VirtioHal;
use crate::sync::UPIntrFreeCell;
use alloc::sync::Arc;
use lazy_static::*;
use polyhal::VIRT_ADDR_START;
use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

const VIRTIO8: usize = 0x10004000;

lazy_static! {
    pub static ref NET_DEVICE: Arc<dyn NetDevice> = Arc::new(VirtIONetWrapper::new());
}

pub trait NetDevice: Send + Sync + Any {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
}

pub struct VirtIONetWrapper(UPIntrFreeCell<VirtIONet<VirtioHal, MmioTransport, 32>>);

impl NetDevice for VirtIONetWrapper {
    fn transmit(&self, data: &[u8]) {
        // self.0
        //     .exclusive_access()
        //     .send(data)
        //     .expect("can't send data")
        todo!("transmit")
    }

    fn receive(&self, data: &mut [u8]) -> usize {
        // self.0
        //     .exclusive_access()
        //     .recv(data)
        //     .expect("can't receive data")
        todo!("receive")
    }
}

impl VirtIONetWrapper {
    pub fn new() -> Self {
        unsafe {
            let virtio = VirtIONet::<VirtioHal, MmioTransport, 32>::new(
                MmioTransport::new(NonNull::new_unchecked(
                    (VIRTIO8 | VIRT_ADDR_START) as *mut VirtIOHeader,
                ))
                .unwrap(),
                512,
            )
            .expect("can't create net device by virtio");
            VirtIONetWrapper(UPIntrFreeCell::new(virtio))
        }
    }
}
