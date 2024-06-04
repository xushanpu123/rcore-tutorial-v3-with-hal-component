use log::{info, trace};

use virtio_drivers::{
    device::{blk::VirtIOBlk, gpu::VirtIOGpu},
    transport::{
        pci::{
            bus::{BarInfo, Cam, Command, DeviceFunction, PciRoot},
            virtio_device_type, PciTransport,
        },
        DeviceType, Transport,
    },
};

use crate::drivers::virtio::VirtioHal;

const MMCONFIG_BASE: usize = 0xB000_0000;

fn enumerate_pci(mmconfig_base: *mut u8, mut f: impl FnMut(&PciTransport) -> bool) -> Option<PciTransport> {
    info!("mmconfig_base = {:#x}", mmconfig_base as usize);

    let mut pci_root = unsafe { PciRoot::new(mmconfig_base, Cam::Ecam) };
    for (device_function, info) in pci_root.enumerate_bus(0) {
        let (status, command) = pci_root.get_status_command(device_function);
        info!(
            "Found {} at {}, status {:?} command {:?}",
            info, device_function, status, command
        );

        if let Some(virtio_type) = virtio_device_type(&info) {
            info!("  VirtIO {:?}", virtio_type);

            // Enable the device to use its BARs.
            pci_root.set_command(
                device_function,
                Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
            );
            dump_bar_contents(&mut pci_root, device_function, 4);

            let mut transport =
                PciTransport::new::<VirtioHal>(&mut pci_root, device_function).unwrap();
            info!(
                "Detected virtio PCI device with device type {:?}, features {:#018x}",
                transport.device_type(),
                transport.read_device_features(),
            );
            if f(&transport) {
                return Some(transport);
            }
        }
    }
    None
}

fn dump_bar_contents(root: &mut PciRoot, device_function: DeviceFunction, bar_index: u8) {
    let bar_info = root.bar_info(device_function, bar_index).unwrap();
    trace!("Dumping bar {}: {:#x?}", bar_index, bar_info);
    if let BarInfo::Memory { address, size, .. } = bar_info {
        let start = address as *const u8;
        unsafe {
            let mut buf = [0u8; 32];
            for i in 0..size / 32 {
                let ptr = start.add(i as usize * 32);
                core::ptr::copy(ptr, buf.as_mut_ptr(), 32);
                if buf.iter().any(|b| *b != 0xff) {
                    trace!("  {:?}: {:x?}", ptr, buf);
                }
            }
        }
    }
    trace!("End of dump");
}

pub fn find_device(f: impl FnMut(&PciTransport) -> bool) -> Option<PciTransport> {
    enumerate_pci(MMCONFIG_BASE as _, f)
}
