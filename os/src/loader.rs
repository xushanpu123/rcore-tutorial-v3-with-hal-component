//! Loading user applications into memory
//!
//! For chapter 3, user applications are simply part of the data included in the
//! kernel binary, so we only need to copy them to the space allocated for each
//! app to load them. We also allocate fixed spaces for each task's
//! [`KernelStack`] and [`UserStack`].
use crate::config::*;
pub use crate::frame_allocater::frame_alloc_persist;
use core::arch::asm;
use log::info;
use polyhal::addr::*;
use polyhal::pagetable::{MappingFlags, MappingSize, PageTable};
#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

/// Get base address of app i.
pub fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_ksp(i: usize) -> usize {
    KERNEL_STACK[i].get_sp()
}

/// Get the total number of applications.
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

#[cfg(target_arch = "x86_64")]
unsafe fn execute_fence() {
    asm!("mfence");
}

#[cfg(target_arch = "riscv64")]
unsafe fn execute_fence() {
    asm!("fence.i");
}

#[cfg(target_arch = "aarch64")]
unsafe fn execute_fence() {
    asm!("dmb ish");
}

#[cfg(target_arch = "loongarch64")]
unsafe fn execute_fence() {
    //asm!("fence");
    //TODO
}

/// Load nth user app at
/// [APP_BASE_ADDRESS + n * APP_SIZE_LIMIT, APP_BASE_ADDRESS + (n+1) * APP_SIZE_LIMIT).
pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    info!("num_app={}", num_app);
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // clear region
        let page_table = PageTable::current();
        let page_num = APP_SIZE_LIMIT / PAGE_SIZE;
        info!("{:#x}", base_i);
        for i in 0..page_num {
            page_table.map_page(
                VirtPage::from_addr(base_i + PAGE_SIZE * i),
                frame_alloc_persist().expect("can't allocate frame"),
                MappingFlags::URWX,
                MappingSize::Page4KB,
            );
        }
        page_table.map_page(
            VirtPage::from_addr(0x1_8000_0000 + i * PAGE_SIZE),
            frame_alloc_persist().expect("can't allocate frame"),
            MappingFlags::URWX,
            MappingSize::Page4KB,
        );
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        info!("start!");
        // load app from data section to memory

        println!("[kernel] Loading app_{}", i);
        info!(
            "app src: {:#x} size: {:#x}",
            app_start[i],
            app_start[i + 1] - app_start[i]
        );
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }

    // the code of the next app into the instruction memory.
    // See also: riscv non-priv spec chapter 3, 'Zifencei' extension.
    unsafe {
        execute_fence();
    }
}
