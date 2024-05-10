use crate::drivers::GPU_DEVICE;
use crate::mm::{MapArea, MapPermission, MapType};
use crate::task::current_process;
use polyhal::addr::PhysAddr;
use polyhal::addr::VirtAddr;
const FB_VADDR: usize = 0x10000000;

pub fn sys_framebuffer() -> isize {
    let fb = GPU_DEVICE.get_framebuffer();
    let len = fb.len();
    // println!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
    let fb_start_pa = PhysAddr::new(fb.as_ptr() as usize);
    let fb_start_vpn = VirtAddr::from(FB_VADDR).floor();

    let current_process = current_process();
    let mut inner = current_process.inner_exclusive_access();
    inner.memory_set.push(
        MapArea::new(
            (FB_VADDR as usize).into(),
            (FB_VADDR + len as usize).into(),
            MapType::Linear(0),
            MapPermission::R | MapPermission::W | MapPermission::U,
        ),
        None,
    );
    FB_VADDR as isize
}

pub fn sys_framebuffer_flush() -> isize {
    GPU_DEVICE.flush();
    0
}
