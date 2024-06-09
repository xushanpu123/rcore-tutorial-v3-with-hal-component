use crate::drivers::bus::virtio::VirtioHal;
use crate::sync::{Condvar, UPIntrFreeCell};
use crate::task::schedule;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::any::Any;
use core::ptr::NonNull;
use polyhal::VIRT_ADDR_START;
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::DeviceType;
use virtio_drivers::transport::Transport;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use polyhal::irq::IRQ;

#[cfg(target_arch = "riscv64")]
const VIRTIO5: usize = 0x10005000 + VIRT_ADDR_START;
#[cfg(target_arch = "aarch64")]
const VIRTIO5: usize = 0x0a003800 + VIRT_ADDR_START;
#[cfg(target_arch = "riscv64")]
const VIRTIO6: usize = 0x10006000 + VIRT_ADDR_START;
#[cfg(target_arch = "aarch64")]
const VIRTIO6: usize = 0x0a003600 + VIRT_ADDR_START;
#[cfg(target_arch = "x86_64")]
const VIRTIO6: usize = 0;
#[cfg(target_arch = "x86_64")]
const VIRTIO5: usize = 0;
// #[cfg(not(target_arch = "x86_64"))]
// type VirtIoTransport = MmioTransport;
// #[cfg(target_arch = "x86_64")]
// type VirtIoTransport = PciTransport;
struct VirtIOInputInner {
    virtio_input: VirtIOInput<VirtioHal, MmioTransport>,
    events: VecDeque<u64>,
}

struct VirtIOInputWrapper {
    inner: UPIntrFreeCell<VirtIOInputInner>,
    condvar: Condvar,
}

pub trait InputDevice: Send + Sync + Any {
    fn read_event(&self) -> u64;
    fn handle_irq(&self);
    fn is_empty(&self) -> bool;
}

lazy_static::lazy_static!(
    pub static ref KEYBOARD_DEVICE: Arc<dyn InputDevice> = Arc::new(VirtIOInputWrapper::new(VIRTIO5));
    pub static ref MOUSE_DEVICE: Arc<dyn InputDevice> = Arc::new(VirtIOInputWrapper::new(VIRTIO6));
);

impl VirtIOInputWrapper {
    pub fn new(addr: usize) -> Self {
        let inner = VirtIOInputInner {
            virtio_input: unsafe {
                VirtIOInput::<VirtioHal, MmioTransport>::new(
                    MmioTransport::new(NonNull::new_unchecked(addr as *mut VirtIOHeader)).unwrap(),
                )
                .unwrap()
            },
            events: VecDeque::new(),
        };
        if addr == VIRTIO5{
            #[cfg(target_arch = "aarch64")]
            IRQ::irq_enable(0x4c);
        }
        if addr == VIRTIO6{
            #[cfg(target_arch = "aarch64")]
            IRQ::irq_enable(0x4b);
        }
        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            condvar: Condvar::new(),
        }
    }
}

impl InputDevice for VirtIOInputWrapper {
    fn is_empty(&self) -> bool {
        self.inner.exclusive_access().events.is_empty()
    }

    fn read_event(&self) -> u64 {
        loop {
            let mut inner = self.inner.exclusive_access();
            if let Some(event) = inner.events.pop_front() {
                return event;
            } else {
                let task_cx_ptr = self.condvar.wait_no_sched();
                drop(inner);
                schedule(task_cx_ptr);
            }
        }
    }

    fn handle_irq(&self) {
        let mut count = 0;
        let mut result = 0;
        self.inner.exclusive_session(|inner| {
            inner.virtio_input.ack_interrupt();
            while let Some(event) = inner.virtio_input.pop_pending_event() {
                count += 1;
                result = (event.event_type as u64) << 48
                    | (event.code as u64) << 32
                    | (event.value) as u64;
                inner.events.push_back(result);
            }
        });
        if count > 0 {
            self.condvar.signal();
        };
    }
}
