use crate::drivers::GPU_DEVICE;
use crate::mm::{MapArea, MapPermission, MapType};
use crate::task::current_process;
use alloc::slice;
use log::info;
use polyhal::addr::VirtAddr;
use polyhal::addr::{PhysAddr, PhysPage, VirtPage};
use polyhal::pagetable::{MappingFlags, MappingSize, PageTable};
use polyhal::{PAGE_SIZE, VIRT_ADDR_START};
const FB_VADDR: usize = 0x10000000;

pub fn sys_framebuffer() -> isize {
    let fb = GPU_DEVICE.get_framebuffer();
    let len = fb.len();
    println!(
        "[kernel] FrameBuffer: addr 0x{:X}, len {}",
        fb.as_ptr() as usize,
        len
    );
    println!("[kernel] vaddr: {:#x}", FB_VADDR);
    let fb_start_pa = PhysAddr::new(fb.as_ptr() as usize);
    let fb_start_vpn = VirtAddr::from(FB_VADDR).floor();

    // println!("");

    let current_process = current_process();
    // inner.memory_set.push(
    //     MapArea::new(
    //         (FB_VADDR as usize).into(),
    //         (FB_VADDR + len as usize).into(),
    //         MapType::Linear(0),
    //         MapPermission::R | MapPermission::W | MapPermission::U,
    //     ),
    //     None,
    // );
    for i in 0..fb.len() / PAGE_SIZE {
        PageTable::current().map_page(
            VirtPage::from_addr(FB_VADDR) + i,
            PhysPage::from_addr(fb.as_ptr() as usize - VIRT_ADDR_START) + i,
            MappingFlags::URWX,
            MappingSize::Page4KB,
        );
    }
    FB_VADDR as isize
}

pub fn sys_framebuffer_flush() -> isize {
    GPU_DEVICE.flush();
    0
}
