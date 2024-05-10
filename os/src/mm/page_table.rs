use core::slice;
use core::str::from_utf8_unchecked;

use alloc::string::{String, ToString};
use polyhal::pagetable::PageTable;

pub fn translated_byte_buffer(_token: PageTable, ptr: *mut u8, len: usize) -> &'static mut [u8] {
    log::trace!("os::mm::page_table::translated_byte_buffer");
    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
}

unsafe fn str_len(ptr: *const u8) -> usize {
    let mut i = 0;
    loop {
        if *ptr.add(i) == 0 {
            break i;
        }
        i += 1;
    }
}

/// Load a string from other address spaces into kernel space without an end `\0`.
pub fn translated_str(_token: PageTable, ptr: *const u8) -> String {
    unsafe {
        let len = str_len(ptr);
        from_utf8_unchecked(slice::from_raw_parts(ptr, len)).to_string()
    }
}

pub fn translated_ref<T>(_token: PageTable, ptr: *const T) -> &'static T {
    unsafe { ptr.as_ref().unwrap() }
}

pub fn translated_refmut<T>(_token: PageTable, ptr: *mut T) -> &'static mut T {
    unsafe { ptr.as_mut().unwrap() }
}
